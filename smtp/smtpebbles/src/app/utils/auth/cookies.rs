use crate::prelude::*;
use crate::app::{
    types::constants::SSS_COOKIE_NAME,
    utils::auth::tokens::verify_session_token,
};
use anyhow::Result;

pub fn get_sss_pub_token(req: &HttpRequest) -> Result<String> {
    let sss_pub_token =
        if let Some(sss_pub_cookie) = req.cookie(SSS_COOKIE_NAME) {
            sss_pub_cookie.value().to_string()
        } else {
            bail!("Session cookie is missing.")
        };

    Ok(sss_pub_token)
}

pub async fn verify_session(
    req: &HttpRequest,
    redis_pool: &deadpool_redis::Pool,
) -> Result<String> {
    let sss_pub_token = match get_sss_pub_token(req) {
        Ok(token) => token,
        Err(e) => bail!(e),
    };

    match verify_session_token(sss_pub_token, &redis_pool).await {
        Ok(session) => Ok(session),
        Err(e) => bail!("{}", e),
    }
}