use std::env;

use actix::prelude::*;
use actix_web::{
    error::ResponseError, guard, http, middleware, web, App, Error,
    HttpRequest, HttpResponse, HttpServer,
};
use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use futures::Future;
use serde_json::json;

use common::db::{AuthData, DbExecutor, QueryRecent};
use common::models::NewBookmark;
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

fn query_recent(
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(QueryRecent(25))
        .from_err()
        .and_then(|res| match res {
            Ok(bookmarks) => Ok(HttpResponse::Ok().json(bookmarks)),
            _ => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn create_bookmark(
    bookmark: web::Json<NewBookmark>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(bookmark.into_inner())
        .from_err()
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
                let token = json!({ "token": token });
                Ok(HttpResponse::Ok().json(token))
            }
            Err(err) => Ok(err.error_response()),
        })
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
        App::new().data(addr.clone()).service(
            web::scope("/api")
                .service(
                    web::resource("auth")
                        // .route(web::get().to_async(whoami))
                        .route(web::post().to_async(login)),
                )
                .service(
                    web::resource("bookmarks")
                        .route(
                            web::post()
                                .guard(guard::fn_guard(admin_guard))
                                .to_async(create_bookmark),
                        )
                        .route(web::get().to_async(query_recent)),
                ),
        )
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
