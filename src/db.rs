use actix::prelude::*;
use bcrypt::verify;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_derive::*;

use crate::error::ServiceError;
use crate::models::{Bookmark, PageData, SlimUser, User};

pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct QueryRecent(pub u32);

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
            .order(created.desc())
            .paginate(1)
            .per_page(1)
            .load_and_count_pages::<Bookmark>(conn)
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
            match verify(&msg.password, &user.password) {
                Ok(matching) => {
                    if matching {
                        return Ok(user.into());
                    }
                }
                Err(_) => (),
            }
        }
        Err(ServiceError::BadRequest(
            "Username and Password don't match".into(),
        ))
    }
}
