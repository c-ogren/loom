use crate::repositories::users::{create_user, get_by_email};
use crate::services::cache::store_cookie;
use crate::services::password::{hash_password, verify_hash};
use crate::state::AppState;

pub async fn register_user(app: &AppState, email: &str, password: &str) -> anyhow::Result<u64> {
    let hashed_password = match hash_password(password) {
        Ok(hashed) => hashed,
        Err(e) => return Err(anyhow::anyhow!("Failed to hash password: {e}")),
    };

    let user_id = create_user(app.pool(), email, hashed_password.as_str()).await?;
    Ok(user_id)
}

pub async fn authenticate_user(
    app: &AppState,
    email: &str,
    password: &str,
) -> anyhow::Result<Option<(u64, String)>> {
    match get_by_email(app.pool(), email).await {
        Ok(Some((id, password_hash))) => {
            if verify_hash(password.as_bytes(), password_hash.as_str()) {
                Ok(Some((id, email.to_string())))
            } else {
                Ok(None)
            }
        }
        Ok(Option::None) => Err(anyhow::anyhow!("User not found")),
        Err(e) => Err(anyhow::anyhow!("Database error: {e}")),
    }
}

pub async fn handle_cookie(app: &AppState, value: &str) -> anyhow::Result<String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    store_cookie(app, &session_id, value).await?;
    Ok(session_id)
}
