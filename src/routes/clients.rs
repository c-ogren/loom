use crate::state::AppState;
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use crate::services::client::register_client_service;

#[derive(Deserialize, Debug)]
pub struct NewClientRequest {
    client_name: String,
    redirect_uris: Vec<String>,
    grant_types: Vec<String>,
    scopes: Vec<String>,
}

#[axum::debug_handler]
pub async fn register_client(
    State(appstate): State<AppState>,
    new_client: Result<Json<NewClientRequest>, JsonRejection>,
) -> impl IntoResponse {
    let new_client = match new_client {
        Ok(Json(client)) => client,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid_json", "detail": err.to_string() })),
            )
                .into_response();
        }
    };

    match register_client_service(
        &appstate,
        new_client.client_name.as_str(),
        &new_client.redirect_uris,
        &new_client.grant_types,
        &new_client.scopes,
    )
    .await
    {
        Ok((client_id, secret_plain)) => {
            info!("Registered new client with ID: {}", client_id);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ 
                  "status": "success",
                  "client_id": client_id,
                  "client_name": new_client.client_name,
                  "client_secret": secret_plain })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "registration_failed", "detail": e.to_string() })),
        )
            .into_response(),
    }
}
