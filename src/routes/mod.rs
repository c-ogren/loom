use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

mod authorize;
mod clients;
mod echo;
mod health;
mod token;
mod user;

pub const COOKIE_NAME: &str = "session_id";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .route("/echo", post(echo::echo))
        .route("/token", post(token::token))
        .route("/authorize", get(authorize::authorize))
        .route("/register", post(user::register_user))
        .route("/clients", post(clients::register_client))
        .route("/login", post(user::login))
}
