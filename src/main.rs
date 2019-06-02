#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::models::Bookmark;

mod models;
mod schema;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    use crate::schema::bookmarks::dsl::*;
    let connection = establish_connection();
    let results = bookmarks
        .limit(10)
        .order(created.desc())
        .load::<Bookmark>(&connection)
        .expect("Error loading posts, lol");

    for post in results {
        println!("{}", post.title);
        println!("{:?}", post.tags);
    }
}
