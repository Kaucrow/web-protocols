use crate::prelude::*;
use crate::app::{ requests, handlers };

#[tracing::instrument(
    name = "Accessing login endpoint"
    skip(db, redis_pool, login_user),
    fields(
        login_user_email = %login_user.email,
    )
)]
#[actix_web::post("/login")]
pub async fn login_user(
    login_user: web::Json<requests::LoginUser>,
    db: web::Data<PgPool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    tracing::info!(target: "app", "Got request");

    match handlers::login_user(login_user.0, db.get_ref(), redis_pool.get_ref()).await {
        Ok(res) => res,
        Err(res) => res.respond_to(&req),
    }
}