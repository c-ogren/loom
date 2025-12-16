use axum::{
    extract::{rejection::QueryRejection, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::services::authorize::AuthCodePayload;
use crate::services::cache::redeem_code;
use crate::services::issue_jwt;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct TokenQuery {
    grant_type: Option<String>,
    redirect_uri: Option<String>,
    code: String,
    client_id: String,
    client_secret: String,
}

#[derive(serde::Serialize)]
pub struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
}

/*
* POST /token?grant_type=...
  &code=...
  &redirect_uri=...
  &client_id=...
  &client_secret=...

    Content-Type: application/x-www-form-urlencoded

* OUTPUT
* 200 { "access_token": "{JWT}", "refresh_token": "{REFRESH_TOKEN}", "token_type": "Bearer", "expires_in": 3600 }

* VALIDATE
* client_id and client_secret match
* authorization code is valid, not expired, not reused
* redirect_uri matches that of the authorization code

* CORE LOGIC
* Issue access token (JWT)
* Issue refresh token (long-lived random string, stored in db)
* Invalidate authorization code (one-time use)
*/

#[axum::debug_handler]
pub async fn token(
    State(app): State<AppState>,
    tq: Result<Query<TokenQuery>, QueryRejection>,
) -> impl IntoResponse {
    let tq = match tq {
        Ok(Query(tq)) => tq,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid_query", "detail": err.to_string() })),
            )
                .into_response();
        }
    };

    //TODO: other grant_types
    let grant_type = tq.grant_type.as_deref().unwrap_or("authorization_code");
    if grant_type != "authorization_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unsupported_grant_type", "detail": "only authorization_code is supported" })),
        )
        .into_response();
    }

    let serialized_payload_option = match redeem_code(&app, tq.code.as_str()).await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "cache_error", "detail": e.to_string() })),
            )
                .into_response();
        }
    };

    let Some(serialized_payload) = serialized_payload_option else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant", "detail": "authorization code not found or already used" })),
        )
            .into_response();
    };

    let d_payload: AuthCodePayload = match serde_json::from_str(&serialized_payload) {
        Ok(sp) => sp,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "invalid_grant", "detail": format!("failed to parse auth code payload: {}", e.to_string()) })),
            )
                .into_response();
        }
    };

    if d_payload.redirect_uri != tq.redirect_uri.as_deref().unwrap_or_default() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_grant", "detail": "redirect_uri does not match" })),
        )
            .into_response();
    }

    if d_payload.client_id != tq.client_id {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_grant", "detail": "client_id does not match" })),
        )
            .into_response();
    }

    let access_token = match issue_jwt(
        &app,
        d_payload.user_id.as_str(),
        &tq.client_id,
        "openid", // TODO: derive scopes from auth code
        tq.client_secret.as_bytes(),
        tq.redirect_uri.unwrap_or_default().as_str(),
    )
    .await
    {
        Ok(access_token) => access_token,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "token_issuance_failed", "detail": e.to_string() })),
            )
                .into_response();
        }
    };

    let response: TokenResponse = TokenResponse {
        access_token,
        refresh_token: "some_refresh_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    };

    Json(json!(response)).into_response()
}
