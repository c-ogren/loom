use crate::{repositories::clients::get_by_client_id, state::AppState};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::services::cache::store_auth_code;

#[derive(Debug)]
pub struct AuthorizeInput {
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub state: Option<String>,
    pub user_id: String,
}

pub struct AuthorizeResult {
    pub code: String,
    pub redirect_uri: String,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthCodePayload {
    pub client_id: String,
    pub user_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub state: Option<String>,
}

fn generate_auth_code() -> String {
    let mut bytes = [0u8; 32]; // 256-bit
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub async fn authorize(
    app: &AppState,
    authorize_input: AuthorizeInput,
) -> Result<AuthorizeResult, anyhow::Error> {
    // client returned here
    let Some(_) = get_by_client_id(app.pool(), &authorize_input).await? else {
        return Err(anyhow::anyhow!("Client not found"));
    };

    let AuthorizeInput {
        client_id,
        user_id,
        redirect_uri,
        scopes,
        state,
    } = authorize_input;

    let code = generate_auth_code();

    let payload = AuthCodePayload {
        client_id,
        user_id,
        redirect_uri: redirect_uri.clone(),
        scopes,
        state: state.clone(),
    };

    store_auth_code(app, &code, serde_json::to_string(&payload)?.as_str()).await?;

    Ok(AuthorizeResult {
        code,
        redirect_uri,
        state: state.unwrap_or_default(),
    })
}
