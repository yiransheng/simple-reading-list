use std::env;

#[macro_use]
extern crate diesel_migrations;

use actix::prelude::*;
use actix_web::{
    body::Body,
    error::ResponseError,
    guard,
    http::{self, header},
    middleware,
    middleware::cors,
    web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection};
use diesel_migrations::embed_migrations;
use dotenv::dotenv;
use futures::{
    future::{self, ok, Either},
    Future,
};
use horrorshow::Template;
use serde_json::json;

use common::config::CONFIG;
use common::db::{AuthData, DbExecutor, QueryRecent};
use common::error::ServiceError;
use common::models::{Bookmark, BookmarkDoc, NewBookmark, PageData, SlimUser};
use common::search::{QueryParser, Search, SearchClient};
use common::templates::{
    bookmark_jsonml, BookmarkItem, IntoBookmark, PageTemplate,
};
use common::utils::{admin_guard, create_token};

embed_migrations!("migrations");

fn create_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    // create db connection pool
    let manager =
        ConnectionManager::<PgConnection>::new(CONFIG.database_url.clone());

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

fn recent_bookmarks(
    page: web::Path<i64>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(QueryRecent(page.into_inner()))
        .from_err()
        .and_then(|res| match res {
            Ok(bookmarks) => {
                let contents: Vec<_> =
                    bookmarks.data.iter().map(bookmark_jsonml).collect();
                let res = PageData {
                    data: contents,
                    total_pages: bookmarks.total_pages,
                    next_page: bookmarks.next_page,
                };
                Ok(HttpResponse::Ok().json(res))
            }
            _ => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn recent_bookmarks_html(
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(QueryRecent(1))
        .from_err()
        .and_then(|res| match res {
            Ok(bookmarks) => {
                let items = bookmarks.data.into_iter().map(BookmarkItem::new);
                let page = PageTemplate::new_with_next_page(
                    bookmarks.next_page,
                    items,
                );
                match page.into_string() {
                    Ok(body) => Ok(HttpResponse::Ok()
                        .content_type("text/html")
                        .body(body)),
                    _ => Ok(HttpResponse::InternalServerError().into()),
                }
            }
            _ => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn search_bookmark(
    search_client: web::Data<SearchClient>,
    search: Option<web::Query<Search>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    match search {
        Some(ref search) if !search.q.is_empty() => Either::A(
            search_client
                .query_docs(QueryParser::new(&search.q).parse())
                .map(move |results| HttpResponse::Ok().json(results)),
        ),
        _ => Either::B(ok(HttpResponse::BadRequest().into())),
    }
}

fn search_bookmark_html(
    search_client: web::Data<SearchClient>,
    search: Option<web::Query<Search>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    #[inline(always)]
    fn redirect_empty_search() -> impl Future<Item = HttpResponse, Error = Error>
    {
        ok(HttpResponse::Found()
            .header(http::header::LOCATION, "/")
            .finish()
            .into_body())
    }
    match search {
        Some(ref search) if !search.q.is_empty() => {
            let query_string = search.q.clone();
            let query = QueryParser::new(&search.q).parse();
            if query.is_empty() {
                return Either::B(redirect_empty_search());
            }
            Either::A(search_client.query_docs(query).and_then(
                move |bookmarks| {
                    let items = bookmarks
                        .docs
                        .into_iter()
                        .map(|doc| BookmarkItem::new(doc.doc));
                    let page =
                        PageTemplate::new_with_query(items, query_string);
                    match page.into_string() {
                        Ok(body) => Ok(HttpResponse::Ok()
                            .content_type("text/html")
                            .body(body)),
                        _ => Ok(HttpResponse::InternalServerError().into()),
                    }
                },
            ))
        }
        _ => Either::B(redirect_empty_search()),
    }
}

fn create_bookmark(
    bookmark: web::Json<NewBookmark>,
    db: web::Data<Addr<DbExecutor>>,
    search_client: web::Data<SearchClient>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(bookmark.into_inner())
        .from_err()
        .and_then::<_, Box<Future<Item = Result<Bookmark, _>, Error = Error>>>(
            move |created| match created {
                Ok(created) => {
                    eprintln!("DB ok");
                    let doc: BookmarkDoc = created.clone().into();
                    let created2 = created.clone();
                    Box::new(
                        search_client
                            .insert_doc(doc)
                            .map(|_| Ok(created))
                            .or_else(|_| future::ok(Ok(created2))),
                    )
                }
                Err(_) => {
                    Box::new(future::ok(Err(ServiceError::InternalServerError)))
                }
            },
        )
        .and_then(move |res| match res {
            Ok(created) => Ok(HttpResponse::Created().json(created)),
            Err(err) => Ok(err.error_response()),
        })
}

fn login(
    auth_data: web::Json<AuthData>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(auth_data.into_inner())
        .from_err()
        .and_then(move |res| match res {
            Ok(user) => {
                let token = create_token(&user)?;
                let res = json!({ "token": token, "user": user });
                Ok(HttpResponse::Ok().json(res))
            }
            Err(err) => Ok(err.error_response()),
        })
}

fn whoami(user: Result<SlimUser, ServiceError>) -> Result<HttpResponse, Error> {
    match user {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Ok(err.error_response()),
    }
}

fn db_migrations(pool: &r2d2::Pool<ConnectionManager<PgConnection>>) {
    let conn: &PgConnection = &pool.get().unwrap();
    embedded_migrations::run_with_output(conn, &mut std::io::stdout())
        .expect("Failed to run migrations");
}

fn main() {
    dotenv().ok();

    let sys = actix_rt::System::new("bookmarks");
    let pool = create_pool();

    db_migrations(&pool);

    // Start 4 parallel db executors
    let addr: Addr<DbExecutor> =
        SyncArbiter::start(4, move || DbExecutor(pool.clone()));
    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(
                cors::Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                    ])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .data(addr.clone())
            .data(SearchClient::new())
            .service(
                web::scope("/api")
                    .service(
                        web::resource("auth")
                            .route(web::get().to(whoami))
                            .route(web::post().to_async(login)),
                    )
                    .service(
                        web::resource("bookmarks:page/{page}")
                            .route(web::get().to_async(recent_bookmarks)),
                    )
                    .service(
                        web::resource("bookmarks").route(
                            web::post()
                                .guard(guard::fn_guard(admin_guard))
                                .to_async(create_bookmark),
                        ),
                    )
                    .service(
                        web::resource("bookmarks/search")
                            .route(web::get().to_async(search_bookmark)),
                    ),
            )
            .service(
                web::resource("/")
                    .route(web::get().to_async(recent_bookmarks_html)),
            )
            .service(
                web::resource("/search")
                    .route(web::get().to_async(search_bookmark_html)),
            )
    })
    .bind((
        "0.0.0.0",
        CONFIG.host_port.parse::<u16>().expect("Bad port"),
    ))
    .unwrap()
    .start();

    println!("Started http server: 0.0.0.0:8080");
    let _ = sys.run();
}
