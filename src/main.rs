#![warn(clippy::pedantic)]
mod macros;
mod middleware;
mod repositories;
mod routes;
mod services;
mod state;

use axum::middleware as _middleware;
use std::net::SocketAddr;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::middleware::log_mw;
use crate::routes::routes;

use crate::state::AppState;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    // load .env if present
    let _ = dotenvy::dotenv();

    // Initialize tracing subscriber (logging)
    let file_appender = tracing_appender::rolling::daily("./logs", "server.log");
    let (nonblocking_writer, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_ansi(false)
        .with_writer(nonblocking_writer)
        .init();
    // End tracing subscriber setup

    let appstate = AppState::new_with_db(2 * 1024 * 1024, 1024).await.unwrap();

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    // Attach trace_layer to Router (not after conversion)
    let router = routes()
        .layer(ConcurrencyLimitLayer::new(
            appstate.get_max_concurrent_requests(),
        ))
        .layer(RequestBodyLimitLayer::new(appstate.get_max_body_bytes()))
        .with_state(appstate)
        .layer(_middleware::from_fn(log_mw));

    // Convert with connect info AFTER layers are set
    let service = router.into_make_service_with_connect_info::<SocketAddr>();

    info!("Server started on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, service).await.unwrap();
}
