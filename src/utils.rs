use actix_web::{
    dev::{Payload, RequestHead},
    FromRequest, HttpRequest,
};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, Header, Validation};
use serde_derive::*;

use crate::config::CONFIG;
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

impl FromRequest for SlimUser {
    type Config = ();
    type Error = ServiceError;
    type Future = Result<SlimUser, ServiceError>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        get_admin(req.head()).ok_or_else(|| ServiceError::Unauthorized)
    }
}

pub fn create_token(data: &SlimUser) -> Result<String, ServiceError> {
    let claims = Claims::from_user(data);
    encode(&Header::default(), &claims, CONFIG.jwt_secret.as_slice())
        .map_err(|_err| ServiceError::InternalServerError)
}

pub fn decode_token(token: &str) -> Result<SlimUser, ServiceError> {
    decode::<Claims>(
        extract_bearer_creds(token)?,
        CONFIG.jwt_secret.as_slice(),
        &Validation::default(),
    )
    .map(|data| {
        let now = Local::now().timestamp();
        if now < data.claims.exp && now >= data.claims.iat {
            Ok(data.claims.into())
        } else {
            Err(ServiceError::Unauthorized)
        }
    })
    .map_err(|_err| ServiceError::Unauthorized)?
}

fn extract_bearer_creds(token: &str) -> Result<&str, ServiceError> {
    const PREFIX: &str = "Bearer ";
    const PREFIX_LEN: usize = 7; // prefix.len() -- not stable yet

    if token.starts_with(PREFIX) {
        token.get(PREFIX_LEN..).ok_or(ServiceError::Unauthorized)
    } else {
        Err(ServiceError::Unauthorized)
    }
}

pub fn admin_guard(req: &RequestHead) -> bool {
    match get_admin(req).filter(|user| user.is_admin) {
        Some(_) => true,
        _ => false,
    }
}

fn get_admin(req: &RequestHead) -> Option<SlimUser> {
    req.headers()
        .get("Authorization")
        .and_then(|token| token.to_str().ok())
        .and_then(|token| decode_token(token).ok())
}
