use actix::prelude::*;
use bcrypt::verify;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use log::*;
use serde_derive::*;

use crate::config::CONFIG;
use crate::error::ServiceError;
use crate::models::{Bookmark, NewBookmark, PageData, SlimUser, User};

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct QueryRecent(pub i64);

#[derive(Debug, Copy, Clone)]
pub struct BookmarkIndexed {
    id: i32,
}
impl BookmarkIndexed {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

impl Message for BookmarkIndexed {
    type Result = Result<Bookmark, diesel::result::Error>;
}

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

impl Message for QueryRecent {
    type Result = Result<PageData<Bookmark>, diesel::result::Error>;
}

impl Handler<QueryRecent> for DbExecutor {
    type Result = Result<PageData<Bookmark>, diesel::result::Error>;

    fn handle(
        &mut self,
        msg: QueryRecent,
        _: &mut Self::Context,
    ) -> Self::Result {
        use crate::pagination::*;
        use crate::schema::bookmarks::dsl::*;

        let conn: &PgConnection = &self.0.get().unwrap();

        bookmarks
            .order_by(created.desc())
            .paginate(msg.0)
            .per_page(20)
            .load_and_count_pages::<Bookmark>(conn)
            .map_err(Into::into)
    }
}

impl Handler<BookmarkIndexed> for DbExecutor {
    type Result = Result<Bookmark, diesel::result::Error>;

    fn handle(
        &mut self,
        msg: BookmarkIndexed,
        _: &mut Self::Context,
    ) -> Self::Result {
        use crate::schema::bookmarks::dsl::*;

        let conn: &PgConnection = &self.0.get().unwrap();

        diesel::update(bookmarks.find(msg.id))
            .set(toshi_index.eq(&CONFIG.toshi_index))
            .get_result::<Bookmark>(conn)
            .map_err(Into::into)
    }
}

impl Message for AuthData {
    type Result = Result<SlimUser, ServiceError>;
}

impl Handler<AuthData> for DbExecutor {
    type Result = Result<SlimUser, ServiceError>;
    fn handle(&mut self, msg: AuthData, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();

        let mut items =
            users.filter(email.eq(&msg.email)).load::<User>(conn)?;

        if let Some(user) = items.pop() {
            if let Ok(true) = verify(&msg.password, &user.password) {
                return Ok(user.into());
            }
        }
        Err(ServiceError::BadRequest(
            "Username and Password don't match".into(),
        ))
    }
}

impl Message for NewBookmark {
    type Result = Result<Bookmark, ServiceError>;
}

impl Handler<NewBookmark> for DbExecutor {
    type Result = Result<Bookmark, ServiceError>;
    fn handle(
        &mut self,
        msg: NewBookmark,
        _: &mut Self::Context,
    ) -> Self::Result {
        use crate::schema::bookmarks::dsl::*;
        let conn: &PgConnection = &self.0.get().unwrap();

        let mut items = diesel::insert_into(bookmarks)
            .values(&msg)
            .get_results(conn)
            .map_err(|err| {
                error!("Create bookmark error: {:?}", err);
                err
            })?;

        items.pop().ok_or_else(|| ServiceError::InternalServerError)
    }
}
