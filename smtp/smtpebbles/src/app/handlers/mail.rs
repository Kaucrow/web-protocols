use crate::prelude::*;
use crate::settings::get_settings;
use crate::app::{
    types::{ requests, error },
    utils::verify_session,
};
use lettre::{
    AsyncTransport,
    Message
};

#[tracing::instrument(
    name = "Handling send email"
    skip(redis_pool, email),
)]
pub async fn send_email(
    email: requests::SendEmail,
    req: &HttpRequest,
    redis_pool: &deadpool_redis::Pool,
) -> Result<HttpResponse, error::SendEmail> {
    let session_email = verify_session(
        req, redis_pool
    )
    .await
    .or(Err(error::SendEmail::Verification))?;

    let settings = get_settings().or(Err(error::SendEmail::Unknown))?;

    if session_email != email.sender {
        return Err(error::SendEmail::InvalidSender(settings.domain));
    }

    let email = build_email(&email).or(Err(error::SendEmail::Unknown))?;

    let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::
        builder_dangerous(settings.smtp.host).port(settings.smtp.port).build();

    mailer.send(email).await.or(Err(error::SendEmail::Unknown))?;

    Ok(HttpResponse::Ok().finish())
}

fn build_email(email: &requests::SendEmail) -> anyhow::Result<Message> {
    let email = Message::builder()
        .from(email.sender.parse()?)
        .to(email.recipient.parse()?)
        .subject(&email.subject)
        .body(email.body.to_string())?;

    Ok(email)
}