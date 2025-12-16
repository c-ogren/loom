use core::result::Result::{Err, Ok};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use std::str;
use time::{Duration, OffsetDateTime};

use crate::repositories::clients::get_by_client_token;
use crate::services::password::verify_hash;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct Claims {
    pub sub: String,   // subject (user id)
    pub aud: String,   // audience (client id)
    pub exp: i64,      // expiration (unix seconds)
    pub iat: i64,      // issued-at
    pub scope: String, // space-delimited scopes
}

#[derive(Debug)]
pub struct TokenInput {
    pub client_id: String,
    pub redirect_uri: String,
}

async fn verify(
    app: &AppState,
    token_input: &TokenInput,
    secret: &[u8],
) -> Result<(), anyhow::Error> {
    match get_by_client_token(app.pool(), token_input).await {
        Ok(Some(client)) => {
            if !verify_hash(secret, client.secret_hash.as_str()) {
                return Err(anyhow::anyhow!("Invalid client secret"));
            }
            Ok(())
        }
        Ok(Option::None) => Err(anyhow::anyhow!("Client not found")),
        Err(e) => Err(anyhow::anyhow!("Database error: {e}")),
    }
}

pub async fn issue_jwt(
    app: &AppState,
    user_id: &str,
    client_id: &str,
    scope: &str,
    secret: &[u8],
    redirect_uri: &str,
) -> anyhow::Result<String> {
    let token_input = TokenInput {
        client_id: client_id.to_string(),
        redirect_uri: redirect_uri.to_string(),
    };

    // verifying secret
    verify(app, &token_input, secret).await?;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let exp = (OffsetDateTime::now_utc() + Duration::hours(1)).unix_timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        aud: client_id.to_string(),
        exp,
        iat: now,
        scope: scope.to_string(),
    };

    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;
    Ok(jwt)
}
