#[macro_use]
extern crate diesel;

use std::env;

use actix::prelude::*;
use actix_web::{
    http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use diesel::prelude::*;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;
use futures::Future;

use crate::db::{DbExecutor, QueryRecent};
use crate::models::JsonData;

mod db;
mod models;
mod schema;

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
            Ok(bookmarks) => {
                Ok(HttpResponse::Ok().json(JsonData::wrap(bookmarks)))
            }
            _ => Ok(HttpResponse::InternalServerError().into()),
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
        App::new()
            .data(addr.clone())
            .service(
                web::scope("/api").service(
                    web::resource("recent")
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
