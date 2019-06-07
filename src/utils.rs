use actix_web::dev::RequestHead;
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde_derive::*;

use crate::error::ServiceError;
use crate::models::SlimUser;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // issuer
    iss: String,
    //issued at
    iat: i64,
    // expiry
    exp: i64,
    // user email
    email: String,
    // admin
    is_admin: bool,
}

impl Claims {
    fn from_user(user: &SlimUser) -> Self {
        Claims {
            iss: "localhost".into(),
            email: user.email.clone(),
            iat: Local::now().timestamp(),
            exp: (Local::now() + Duration::hours(24)).timestamp(),
            is_admin: user.is_admin,
        }
    }
}

impl From<Claims> for SlimUser {
    fn from(claims: Claims) -> Self {
        SlimUser {
            email: claims.email,
            is_admin: claims.is_admin,
        }
    }
}

pub fn create_token(data: &SlimUser) -> Result<String, ServiceError> {
    let claims = Claims::from_user(data);
    encode(&Header::default(), &claims, get_secret().as_ref())
        .map_err(|_err| ServiceError::InternalServerError)
}

pub fn decode_token(token: &str) -> Result<SlimUser, ServiceError> {
    decode::<Claims>(
        extract_bearer_creds(token)?,
        get_secret().as_ref(),
        &Validation::default(),
    )
    .map(|data| Ok(data.claims.into()))
    .map_err(|_err| ServiceError::Unauthorized)?
}

fn extract_bearer_creds(token: &str) -> Result<&str, ServiceError> {
    const PREFIX: &'static str = "Bearer ";
    const PREFIX_LEN: usize = 7; // prefix.len() -- not stable yet

    if token.starts_with(PREFIX) {
        token.get(PREFIX_LEN..).ok_or(ServiceError::Unauthorized)
    } else {
        Err(ServiceError::Unauthorized)
    }
}

pub fn admin_guard(req: &RequestHead) -> bool {
    req.headers()
        .get("Authorization")
        .and_then(|token| token.to_str().ok())
        .and_then(|token| decode_token(token).ok())
        .map(|user| user.is_admin)
        .unwrap_or(false)
}

fn get_secret() -> String {
    std::env::var("JWT_SECRET").expect("Missing jwt secret env var")
}
