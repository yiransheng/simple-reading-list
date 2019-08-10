use std::convert::TryFrom;
use std::env;
use std::error::Error;

use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use dotenv::dotenv;
use structopt::StructOpt;

use common::models::NewUser;

#[derive(StructOpt, Debug)]
#[structopt(name = "create-admin-user")]
struct AdminOpt {
    #[structopt(short = "u", long = "user")]
    user: String,

    #[structopt(short = "p", long = "password")]
    password: String,
}

impl<'a> TryFrom<&'a mut AdminOpt> for NewUser<'a> {
    type Error = bcrypt::BcryptError;

    fn try_from(opt: &'a mut AdminOpt) -> Result<Self, Self::Error> {
        opt.password = hash(&opt.password, DEFAULT_COST)?;

        Ok(NewUser {
            email: &opt.user,
            password: &opt.password,
            is_admin: true,
        })
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn create_admin_user<'a>(
    conn: &PgConnection,
    user: NewUser<'a>,
) -> Result<(), diesel::result::Error> {
    use common::schema::users::dsl::*;

    let _ = diesel::insert_into(users)
        .values(&user)
        .on_conflict(email)
        .do_update()
        .set(&user)
        .execute(conn)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut opt = AdminOpt::from_args();
    assert!(!opt.user.is_empty());
    assert!(!opt.password.is_empty());

    let user = NewUser::try_from(&mut opt)?;

    let conn = establish_connection();

    create_admin_user(&conn, user)?;

    Ok(())
}
