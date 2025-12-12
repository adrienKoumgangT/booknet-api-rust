use tokio::net::TcpListener;
use crate::main_app::build_app;
use crate::shared::configuration::AppConfig;
use crate::shared::logging::log;

mod shared;
mod services;
mod main_app;
mod metrics_app;

async fn start_main_server(config: AppConfig) {
    let app = build_app(config).await.unwrap();

    tracing::info!("Server running on http://{}", app.addr);
    log::info2(&format!("Server running on http://{}", app.addr));
    tracing::info!("Swagger UI available at http://{}/swagger-ui", app.addr);
    log::info2(&format!("Swagger UI available at http://{}/swagger-ui", app.addr));

    let listener = TcpListener::bind(app.addr).await.unwrap();
    axum::serve(listener, app.router).await.unwrap();
}

async fn start_metrics_server(config: AppConfig) {
    let app = metrics_app::build_metrics_app().await;

    tracing::info!("Metrics available at http://{}/metrics", config.metrics_addr);
    log::info2(&format!("Metrics available at http://{}/metrics", config.metrics_addr));

    let listener = TcpListener::bind(config.metrics_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // async fn main() {

    // Initialize tracing
    // tracing_subscriber::fmt::init();
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // Load configuration
    let config = AppConfig::default()?;

    log::init_from_config(config.is_prod);

    let (_main_server, _metrics_server) = tokio::join!(start_main_server(config.clone()), start_metrics_server(config.clone()));

    Ok(())
}
