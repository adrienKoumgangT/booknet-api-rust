use std::future::ready;
use axum::Router;
use axum::routing::get;
use crate::shared::metrics::prometheus::setup_metrics_recorder;

pub async fn build_metrics_app() -> Router {
    let recorder_handle = setup_metrics_recorder();
    
    Router::new().route("/metrics", get(move || ready(recorder_handle.render())))
}
