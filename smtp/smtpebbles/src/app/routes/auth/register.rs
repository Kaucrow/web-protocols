use crate::prelude::*;
use crate::app::{ requests, handlers };

#[tracing::instrument(
    name = "Accessing register endpoint"
    skip(db, new_user),
    fields(
        new_user_email = %new_user.email,
    )
)]
#[actix_web::post("/register")]
pub async fn register_user(
    new_user: web::Json<requests::NewUser>,
    db: web::Data<PgPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    tracing::info!(target: "app", "Got request");

    match handlers::register_user(new_user.0, db.get_ref()).await {
        Ok(res) => res,
        Err(res) => res.respond_to(&req),
    }
}