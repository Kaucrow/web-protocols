use crate::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Auth {
    #[error("A user with the provided email already exists")]
    UserAlreadyExists,
    #[error("Could not find a user with the provided email")]
    UserNotFound,
    #[error("An unknown error was produced")]
    Unknown,
}

impl Responder for Auth {
    type Body = BoxBody;
    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse {
        match self {
            Auth::UserAlreadyExists => HttpResponse::Conflict().body(self.to_string()), // HTTP 409
            Auth::UserNotFound => HttpResponse::NotFound().body(self.to_string()),      // HTTP 404
            Auth::Unknown => HttpResponse::InternalServerError().finish(),              // HTTP 500
        }
    }
}