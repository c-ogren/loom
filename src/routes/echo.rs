use crate::state::AppState;
use axum::extract::ConnectInfo;
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{error, info};

#[derive(serde::Deserialize)]
pub struct EchoRequest {
    data: Vec<u8>,
}

fn json_parse_error(err: &JsonRejection) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "status": "error",
            "error": "invalid_json",
            "detail": err.to_string()
        })),
    )
}

#[axum::debug_handler]
// a funny handler where we echo back the data sent to us- converting Vec<u8> to String
pub async fn echo(
    State(appstate): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    payload: Result<Json<EchoRequest>, JsonRejection>,
) -> impl IntoResponse {
    match payload {
        Ok(Json(mut p)) => {
            info!("Echoing back {} bytes to client {addr}", p.data.len());
            appstate.add_bytes(p.data.len() as u64);
            appstate.increment_requests();
            p.data.push(b'!');
            match String::from_utf8(p.data) {
                Ok(s) => {
                    info!("Successfully echoed back to client {}: {}", addr, s);
                    Json(json!({"status": "success", "data": s})).into_response()
                }
                Err(e) => {
                    error!("UTF-8 conversion error for client {}: {}", addr, e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": e.to_string()})),
                    )
                        .into_response()
                }
            }
        }
        Err(err) => json_parse_error(&err).into_response(),
    }
}
