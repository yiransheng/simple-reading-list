use std::env;

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
use dotenv::dotenv;
use futures::{
    future::{self, ok, Either},
    Future,
};
use horrorshow::Template;
use serde_json::json;

use common::db::{AuthData, DbExecutor, QueryRecent};
use common::error::ServiceError;
use common::models::{Bookmark, BookmarkDoc, NewBookmark, SlimUser};
use common::search::{Search, SearchClient};
use common::templates::{BookmarkItem, IntoBookmark, PageTemplate};
use common::utils::{admin_guard, create_token};

fn create_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

fn recent_bookmarks(
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(QueryRecent(25))
        .from_err()
        .and_then(|res| match res {
            Ok(bookmarks) => Ok(HttpResponse::Ok().json(bookmarks)),
            _ => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn recent_bookmarks_html(
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(QueryRecent(25))
        .from_err()
        .and_then(|res| match res {
            Ok(bookmarks) => {
                let items = bookmarks.data.into_iter().map(BookmarkItem::new);
                let page = PageTemplate::new(items);
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
                .query_docs(dbg!(&search.q))
                .map(move |results| HttpResponse::Ok().json(results)),
        ),
        _ => Either::B(ok(HttpResponse::BadRequest().into())),
    }
}

fn search_bookmark_html(
    search_client: web::Data<SearchClient>,
    search: Option<web::Query<Search>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    match search {
        Some(ref search) if !search.q.is_empty() => {
            Either::A(search_client.query_docs(&search.q).and_then(
                move |bookmarks| {
                    let items = bookmarks
                        .docs
                        .into_iter()
                        .map(|doc| BookmarkItem::new(doc.doc));
                    let page = PageTemplate::new(items);
                    match page.into_string() {
                        Ok(body) => Ok(HttpResponse::Ok()
                            .content_type("text/html")
                            .body(body)),
                        _ => Ok(HttpResponse::InternalServerError().into()),
                    }
                },
            ))
        }
        _ => Either::B(ok(HttpResponse::Found()
            .header(http::header::LOCATION, "/")
            .finish()
            .into_body())),
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
                    Box::new(
                        search_client.insert_doc(doc).map(move |_| Ok(created)),
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

fn main() {
    dotenv().ok();

    let sys = actix_rt::System::new("bookmarks");
    let pool = create_pool();
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
                        web::resource("bookmarks")
                            .route(
                                web::post()
                                    // .guard(guard::fn_guard(admin_guard))
                                    .to_async(create_bookmark),
                            )
                            .route(web::get().to_async(recent_bookmarks)),
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
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
