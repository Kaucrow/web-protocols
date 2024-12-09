use crate::prelude::*;
use crate::app::requests;

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
) -> HttpResponse {
    tracing::info!(target: "app", "Got request");

    let user_res = query("
        SELECT * FROM users WHERE email = $1
    ")
    .bind(&new_user.email)
    .fetch_optional(db.get_ref())
    .await;

    if let Ok(user_res) = user_res {
        if user_res.is_some() {
            return HttpResponse::Conflict().json("A user with the provided email already exists.")
        }
    } else {
        return HttpResponse::InternalServerError().finish();
    }

    let res = query("
        INSERT INTO users
        (email, name, password)
        VALUES ($1, $2, $3)
    ")
    .bind(&new_user.email)
    .bind(&new_user.name)
    .bind(&new_user.password)
    .execute(db.get_ref())
    .await;

    if let Ok(_) = res {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}