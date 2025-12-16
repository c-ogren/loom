use axum::middleware::Next;
use axum::{body::Body, extract::ConnectInfo, http::Request, response::Response};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::info;

// Custom logging middleware
pub async fn log_mw(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let client = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map_or_else(|| "unknown".into(), |ci| ci.0.to_string());

    let res = next.run(req).await;

    let sensitive_fields = ["code", "client_secret"];

    let v_query_fields: Vec<&str> = uri.query().unwrap_or("").split('&').collect();

    let scrubbed_query: Vec<String> = v_query_fields
        .into_iter()
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let mut kv = pair.splitn(2, '=');
            let key = kv.next().unwrap_or("");
            let value = kv.next().unwrap_or("");
            if sensitive_fields.contains(&key) {
                format!("{key}=<scrubbed>")
            } else {
                format!("{key}={value}")
            }
        })
        .collect();

    let scrubbed_query_str = scrubbed_query.join("&");
    // Printing to stdout for docker logs
    println!(
        "method: {} || uri_path: {} || query: {} || client: {} || status: {} || latency_ms: {}",
        method,
        uri.path(),
        scrubbed_query_str,
        client,
        res.status(),
        start.elapsed().as_millis()
    );
    info!(
        method=%method,
        uri_path=%uri.path(),
        query=%scrubbed_query_str,
        client=%client,
        status=%res.status(),
        latency_ms=%start.elapsed().as_millis(),
        "http"
    );
    res
}
