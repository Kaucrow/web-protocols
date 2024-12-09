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

#[derive(Debug, Error)]
pub enum SendEmail {
    #[error("The email sender is not registered in {0}")]
    InvalidSender(String),
    #[error("Verification failed")]
    Verification,
    #[error("An unknown error was produced")]
    Unknown,
}

impl Responder for SendEmail {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            SendEmail::InvalidSender(_) => HttpResponse::BadRequest().body(self.to_string()),
            SendEmail::Verification => HttpResponse::Unauthorized().body(self.to_string()),
            SendEmail::Unknown => HttpResponse::InternalServerError().finish(),
        } 
    }
}