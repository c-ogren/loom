use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tracing::info;

#[axum::debug_handler]
pub async fn health_check(State(appstate): State<AppState>) -> impl IntoResponse {
    info!("Health check requested");
    (StatusCode::OK, appstate.to_string())
}
