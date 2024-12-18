use crate::prelude::*;
use crate::app::{ requests, handlers };

#[tracing::instrument(
    name = "Accessing send email endpoint"
    skip(redis_pool, email),
)]
#[actix_web::post("/send")]
pub async fn send_email(
    email: web::Json<requests::SendEmail>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    tracing::info!(target: "app", "Got request");

    match handlers::send_email(email.0, &req, redis_pool.get_ref()).await {
        Ok(res) => res,
        Err(res) => res.respond_to(&req),
    }
}