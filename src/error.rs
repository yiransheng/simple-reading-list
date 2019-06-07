use std::convert::From;

use actix_web::{error::ResponseError, HttpResponse};
use bcrypt::BcryptError;
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error};

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            ServiceError::InternalServerError => {
                HttpResponse::InternalServerError()
                    .json("Internal Server Error, Please try later")
            }
            ServiceError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(message)
            }
            ServiceError::Unauthorized => {
                HttpResponse::Unauthorized().json("Unauthorized")
            }
        }
    }
}

impl From<Error> for ServiceError {
    fn from(error: Error) -> ServiceError {
        // Right now we just care about UniqueViolation from diesel
        // But this would be helpful to easily map errors as our app grows
        match error {
            Error::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info
                        .details()
                        .unwrap_or_else(|| info.message())
                        .to_string();
                    return ServiceError::BadRequest(message);
                }
                ServiceError::InternalServerError
            }
            _ => ServiceError::InternalServerError,
        }
    }
}

impl From<BcryptError> for ServiceError {
    fn from(error: BcryptError) -> ServiceError {
        match error {
            BcryptError::InvalidPassword => ServiceError::Unauthorized,
            _ => ServiceError::InternalServerError,
        }
    }
}
