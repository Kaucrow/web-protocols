use crate::prelude::*;
use crate::settings::get_settings;
use crate::app::{
    utils::get_redis_conn,
    types::{
        constants::{ SSS_PUB_TK, SSS_DATA_TK },
        postgres::User,
    }
};
use anyhow::Result;

/// Store the session key prefix as a const so it can't be typo'd anywhere it's used.
const SESSION_KEY_PREFIX: &str = "session_";

/// Issues a PASETO token to a user for storing the session.
/// Returns the session UUID token which should be set as a cookie,
/// and sets a key-value pair in Redis where this UUID is the key
/// and the session token is the value. This token has the user's id encoded.
#[tracing::instrument(name = "Issue PASETO token for session uuid", skip(redis_pool, user))]
pub async fn issue_session_token(
    user: User,
    redis_pool: &deadpool_redis::Pool,
) -> Result<String> {
    let settings = get_settings()?;

    let sss_uuid = Uuid::new_v4();

    let mut redis_conn = get_redis_conn(redis_pool).await?;

    let redis_key = format!("{}{}", SESSION_KEY_PREFIX, sss_uuid);

    // Build the redis token containing the user id.
    let redis_token = build_sss_data_token(&settings, user).await?;

    redis_conn.set_ex::<_, _, ()>(redis_key, redis_token, settings.app.secret.session_token_expiration * 60).await?;

    // Build the session token to be set as a cookie, containing the UUID for the redis session key.
    let sss_token = {
        let mut claims = Claims::new()?;

        claims.add_additional(SSS_PUB_TK.uuid_key, json!(sss_uuid))?;

        // Use the 256 bit secret key as the symmetric key 
        let sk = SymmetricKey::<V4>::from(settings.app.secret.key.as_bytes())?;

        local::encrypt(
            &sk,
            &claims,
            None,
            Some(settings.app.secret.hmac.as_bytes()),
        )?
    };

    tracing::debug!(target: "backend", "Finished.");
    Ok(sss_token)
}

// Build the redis token containing the user session data.
async fn build_sss_data_token(
    settings: &crate::settings::Settings,
    user: User,
) -> Result<String> {
    let mut claims = Claims::new()?;

    claims.add_additional(SSS_DATA_TK.session_key, user.email)?;

    // Use the 256 bit secret key as the symmetric key 
    let sk = SymmetricKey::<V4>::from(settings.app.secret.key.as_bytes())?;

    Ok(local::encrypt(
        &sk,
        &claims,
        None,
        Some(settings.app.secret.hmac.as_bytes()),
    )?)
}

/// Retrieves the session UUID from the session uuid token, and
/// uses it to retrieve the session token from redis, where the
/// key is the session key prefix plus the UUID.
/// Returns the user id.
#[tracing::instrument(name = "Verify PASETO token for session uuid", skip(redis_pool))]
pub async fn verify_session_token(
    sss_pub_token: String,
    redis_pool: &deadpool_redis::Pool,
) -> Result<String> {
    let settings = get_settings()?;

    let claims = get_token_claims(&settings, sss_pub_token)?;

    let sss_uuid_claim = get_claim(&claims, SSS_PUB_TK.uuid_key);
    let sss_uuid: Uuid = serde_json::from_value(sss_uuid_claim.clone())?;

    let mut redis_conn = get_redis_conn(redis_pool).await?;

    let redis_key = format!("{}{}", SESSION_KEY_PREFIX, sss_uuid);
    let sss_data_token: Option<String> = redis_conn.get(redis_key.clone()).await?;

    if let Some(sss_data_token) = sss_data_token {
        redis_conn.expire::<_, ()>(redis_key, (settings.app.secret.session_token_expiration * 60) as i64).await?;

        let claims = get_token_claims(&settings, sss_data_token)?;

        let session_claim = get_claim(&claims, SSS_DATA_TK.session_key);
        let email: String = serde_json::from_value(session_claim.clone())?;

        Ok(email)
    } else {
        bail!("The session is expired");
    }
}

pub fn get_token_claims(settings: &crate::settings::Settings, token: String) -> Result<Claims> {
    let sk = SymmetricKey::<V4>::from(settings.app.secret.key.as_bytes()).unwrap();

    let validation_rules = ClaimsValidationRules::new();

    let untrusted_token = UntrustedToken::<Local, V4>::try_from(&token)
        .map_err(|e| anyhow!(format!("TokenValidation: {}", e)))?;

    let trusted_token = local::decrypt(
        &sk,
        &untrusted_token,
        &validation_rules,
        None,
        Some(settings.app.secret.hmac.as_bytes()),
    )
    .map_err(|e| anyhow!(format!("PASETO: {}", e)))?;

    let claims = trusted_token.payload_claims().unwrap();

    Ok(claims.clone())
}

fn get_claim<'a>(claims: &'a Claims, str: &str) -> &'a serde_json::value::Value {
    let value = claims.get_claim(str)
        .expect(format!("Could not find `{}` in claims.", str).as_str());

    value
}