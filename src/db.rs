use actix::prelude::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::models::{Bookmark, PageData};

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct QueryRecent(pub u32);

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
            .order(created.desc())
            .paginate(1)
            .per_page(1)
            .load_and_count_pages::<Bookmark>(conn)
            .map_err(Into::into)
    }
}
