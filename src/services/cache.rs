use crate::state::AppState;
use crate::{redis_get, redis_getdel, redis_set_ex};

static AUTH_CODE_EXPIRATION_SECS: u64 = 10 * 60; // 10 minutes
static ACCESS_TOKEN_EXPIRATION_SECS: u64 = 60 * 60; // 1 hour

pub async fn store_auth_code(app: &AppState, code: &str, payload: &str) -> anyhow::Result<()> {
    let mut conn = app.redis_client().get_async_connection().await?;
    redis_set_ex!(conn, "auth_code", code, payload, AUTH_CODE_EXPIRATION_SECS);
    Ok(())
}

pub async fn store_cookie(app: &AppState, name: &str, value: &str) -> anyhow::Result<()> {
    let mut conn = app.redis_client().get_async_connection().await?;
    redis_set_ex!(conn, "cookie", name, value, ACCESS_TOKEN_EXPIRATION_SECS);
    Ok(())
}

pub async fn redeem_code(app: &AppState, code: &str) -> anyhow::Result<Option<String>> {
    let mut conn = app.redis_client().get_async_connection().await?;
    let v = redis_getdel!(conn, "auth_code", code);
    Ok(v)
}

pub async fn get_cookie(app: &AppState, name: &str) -> anyhow::Result<Option<String>> {
    let mut conn = app.redis_client().get_async_connection().await?;
    let v = redis_get!(conn, "cookie", name);
    Ok(v)
}
