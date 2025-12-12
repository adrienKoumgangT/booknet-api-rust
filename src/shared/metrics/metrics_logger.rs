use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{Method, StatusCode},
};
use std::time::Instant;

use crate::shared::logging::log;
// use crate::shared::metrics::prometheus::Metrics;

pub async fn metrics_and_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Start timer for metrics
    let start_time = Instant::now();

    // Log request
    log::info2(&format!("[HTTP] [{}] {}", method, uri));

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start_time.elapsed();
    let status = response.status();

    // Record metrics (if you have metrics setup)
    // state.metrics.record_http_request(method.as_str(), &path, status.as_u16(), duration.as_secs_f64());

    // Log response with color based on status code
    match status.as_u16() {
        200..=299 => {
            log::success(&format!(
                "[HTTP] [{}] {} - {}ms",
                method,
                uri,
                duration.as_millis()
            ));
        },
        300..=399 => {
            log::info(&format!(
                "[HTTP] [{}] {} - {}ms",
                method,
                uri,
                duration.as_millis()
            ));
        },
        400..=499 => {
            log::warning(&format!(
                "[HTTP] [{}] {} - {}ms",
                method,
                uri,
                duration.as_millis()
            ));
        },
        500..=599 => {
            log::error(&format!(
                "[HTTP] [{}] {} - {}ms",
                method,
                uri,
                duration.as_millis()
            ));
        },
        _ => {
            log::debug(&format!(
                "[HTTP] [{}] {} - {}ms",
                method,
                uri,
                duration.as_millis()
            ));
        }
    }
    

    response
}