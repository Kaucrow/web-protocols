use crate::prelude::*;
use crate::app::{
    utils::auth::tokens::issue_session_token,
    requests,
    types::{ self, error, constants::SSS_COOKIE_NAME }
};

#[tracing::instrument(
    name = "Handling user login"
    skip(db, user),
)]
pub async fn login_user(
    user: requests::LoginUser,
    db: &PgPool,
    redis_pool: &deadpool_redis::Pool,
) -> Result<HttpResponse, error::Auth> {
    let user_res = query("
        SELECT * FROM users WHERE email = $1
    ")
    .bind(&user.email)
    .fetch_optional(db)
    .await;

    if let Ok(user_res) = user_res {
        if user_res.is_none() {
            return Err(error::Auth::UserNotFound);
        }

        let user = types::postgres::User::try_from(
            user_res.unwrap()
        )
        .or(Err(error::Auth::Unknown))?;

        let res = issue_session_token(user, redis_pool).await;

        if let Ok(sss_pub_token) = res {
            let session_cookie = Cookie::build(SSS_COOKIE_NAME, sss_pub_token.to_string())
                .path("/")
                .http_only(true)
                .finish();

            Ok(HttpResponse::Ok().cookie(session_cookie).finish())
        } else {
            Err(error::Auth::Unknown)
        }
    } else {
        return Err(error::Auth::Unknown);
    }
}

pub async fn register_user(
    user: requests::NewUser,
    db: &PgPool
) -> Result<HttpResponse, error::Auth> {
    let user_res = query("
        SELECT * FROM users WHERE email = $1
    ")
    .bind(&user.email)
    .fetch_optional(db)
    .await;

    if let Ok(user_res) = user_res {
        if user_res.is_some() {
            return Err(error::Auth::UserAlreadyExists);
        }
    } else {
        return Err(error::Auth::Unknown);
    }

    let res = query("
        INSERT INTO users
        (email, name, password)
        VALUES ($1, $2, $3)
    ")
    .bind(&user.email)
    .bind(&user.name)
    .bind(&user.password)
    .execute(db)
    .await;

    if res.is_ok() {
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(error::Auth::Unknown)
    }
}