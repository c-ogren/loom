use crate::{routes::COOKIE_NAME, services::cache::get_cookie, state::AppState};
use axum::{
    Json, extract::{Query, State, rejection::QueryRejection}, http::StatusCode, response::IntoResponse
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::services::{authorize_svc, AuthorizeInput};

#[derive(Deserialize, Debug)]
pub struct AuthorizeQuery {
    client_id: Option<String>,
    response_type: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
}

#[derive(Serialize)]
struct AuthorizeResponse {
    redirect_to: String,
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

/*
* GET /authorize?client_id=...
    &response_type=...
    &redirect_uri=...
    &scope=...
    &state=...

* OUTPUT
* Redirect to:
* 302 ${redirect_uri}?code=AUTH_CODE&state=STATE
*
* CORE LOGIC
* Create an authorization code (short-lived, 5-10 min), store it in db, bind it to client, user, redirect_uri
*/

#[axum::debug_handler]
pub async fn authorize(
    State(app): State<AppState>,
    jar: CookieJar,
    aq: Result<Query<AuthorizeQuery>, QueryRejection>,
) -> impl IntoResponse {
    let aq = match aq {
        Ok(Query(aq)) => aq,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid_query", "detail": err.to_string() })),
            ).into_response();
        }
    };

    let Some(client_id) = aq.client_id.as_deref() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_request", "detail": "client_id is required" })),
        ).into_response();
    };

    let Some(rt) = aq.response_type.as_deref() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_request", "detail": "response_type is required" })),
        ).into_response();
    };

    if rt != "code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unsupported_response_type", "detail": "only response_type=code is supported" })),
        ).into_response();
    }

    let redirect_uri = if let Some(uri) = aq.redirect_uri.as_deref() {
        uri.to_string()
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_request", "detail": "redirect_uri is required for this client" })),
        ).into_response();
    };

    let requested_scopes: Vec<&str> = aq
        .scope
        .as_deref()
        .map_or_else(Vec::new, |s| s.split_whitespace().collect());

    let scopes = requested_scopes
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<_>>();

    let cookie = match jar.get(COOKIE_NAME) {
        Some(c) => c.value(),
        Option::None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized", "detail": "user not logged in" })),
            ).into_response();
        }
    };

    let email = match get_cookie(&app, cookie).await {
        Ok(Some(e)) => e,
        Ok(Option::None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "unauthorized", "detail": "invalid session" })),
            ).into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "server_error", "detail": err.to_string() })),
            ).into_response();
        }
    };

    match authorize_svc(
        &app,
        AuthorizeInput {
            client_id: client_id.to_string(),
            redirect_uri,
            scopes,
            user_id: email,
            state: aq.state,
        },
    )
    .await
    {
        Ok(res) => (
            StatusCode::FOUND,
            Json(AuthorizeResponse {
                redirect_to: format!("{}?code={}", res.redirect_uri, res.code),
                code: res.code,
                state: if res.state.is_empty() {
                    None
                } else {
                    Some(res.state)
                },
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "authorization_failed", "detail": err.to_string() })),
        ).into_response(),
    }
}
