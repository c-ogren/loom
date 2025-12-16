use crate::{routes::COOKIE_NAME, state::AppState};
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use tracing::info;

use crate::services::user::{
    authenticate_user, handle_cookie, register_user as register_user_service,
};

#[derive(serde::Deserialize)]
pub struct UserRequest {
    email: String,
    password: String,
}

#[axum::debug_handler]
pub async fn register_user(
    State(appstate): State<AppState>,
    new_user: Result<Json<UserRequest>, JsonRejection>,
) -> impl IntoResponse {
    let new_user = match new_user {
        Ok(Json(user)) => user,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_json", "detail": err.to_string() })),
            ).into_response();
        }
    };

    match register_user_service(
        &appstate,
        new_user.email.as_str(),
        new_user.password.as_str(),
    )
    .await
    {
        Ok(user_id) => {
            info!("Registered new user with ID: {}", user_id);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "status": "success", "user_id": user_id })),
            ).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "registration_failed", "detail": e.to_string() })),
        ).into_response(),
    }
}

/*
 * LOGIN
 */
#[axum::debug_handler]
pub async fn login(
    State(app): State<AppState>,
    _: CookieJar,
    user: Result<Json<UserRequest>, JsonRejection>,
) -> impl IntoResponse {
    let user = match user {
        Ok(Json(user)) => user,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_json", "detail": err.to_string() })),
            )
                .into_response();
        }
    };

    let Some((user_id, user_email)) = (match authenticate_user(
        &app,
        user.email.as_str(),
        user.password.as_str(),
    )
    .await
    {
        Ok(user) => user,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "authentication_failed", "detail": err.to_string() })),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid_credentials", "detail": "email or password is incorrect" })),
        )
            .into_response();
    };

    info!("User logged in: id={}, email={}", user_id, user_email);

    let session_id = match handle_cookie(&app, user_email.as_str()).await {
        Ok(sid) => sid,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "session_store_failure", "detail": err.to_string() })),
            )
                .into_response();
        }
    };
    info!("Set session cookie for user id={}: {}", user_id, session_id);
    let mut cookie = Cookie::new(COOKIE_NAME, session_id);
    cookie.set_http_only(true);
    cookie.set_secure(false);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    // TODO: set domain if needed with cookie.set_domain("example.com");

    let jar = CookieJar::new().add(cookie);

    (
        jar,
        (StatusCode::OK,
        Json(serde_json::json!({ "status": "success", "message": "User authenticated successfully" })))
    )
        .into_response()
}
