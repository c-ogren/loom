use crate::repositories::clients::create_client;
use crate::services::password::hash_password;
use crate::state::AppState;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::{rngs::OsRng, RngCore};

pub async fn register_client_service(
    app: &AppState,
    client_name: &str,
    redirect_uris: &[String],
    grant_types: &[String],
    scopes: &[String],
) -> anyhow::Result<(u64, String)> {
    let mut secret_bytes = [0u8; 32]; // 256-bit
    OsRng.fill_bytes(&mut secret_bytes);
    let secret_plain = URL_SAFE_NO_PAD.encode(secret_bytes);
    let secret_hash = hash_password(&secret_plain)
        .map_err(|e| anyhow::anyhow!("Failed to hash client secret: {e}"))?;

    let client_id = create_client(
        app.pool(),
        client_name,
        secret_hash.as_str(),
        redirect_uris,
        grant_types,
        scopes,
    )
    .await?;

    Ok((client_id, secret_plain))
}
