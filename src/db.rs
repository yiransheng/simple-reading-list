use actix::prelude::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::models::Bookmark;

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct QueryRecent(pub u32);

impl Message for QueryRecent {
    type Result = Result<Vec<Bookmark>, diesel::result::Error>;
}

impl Handler<QueryRecent> for DbExecutor {
    type Result = Result<Vec<Bookmark>, diesel::result::Error>;

    fn handle(
        &mut self,
        msg: QueryRecent,
        _: &mut Self::Context,
    ) -> Self::Result {
        use crate::schema::bookmarks::dsl::*;

        let conn: &PgConnection = &self.0.get().unwrap();

        bookmarks
            .limit(msg.0 as i64)
            .order(created.desc())
            .load::<Bookmark>(conn)
            .map_err(Into::into)
    }
}
