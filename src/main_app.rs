use axum::{middleware, Router, routing::get, http::Method};
use std::net::SocketAddr;
use std::str::FromStr;
use tower_http::{trace::TraceLayer, cors::{Any, CorsLayer}, compression::CompressionLayer};

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::shared::configuration::AppConfig;
use crate::shared::openapi::spec::ApiDoc;
use crate::route::{metadata_route};
use crate::shared::state::AppState;
use crate::shared::metrics::metrics_logger::metrics_and_logging_middleware;

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/metadata", metadata_route::routes())
}

pub struct App { pub addr: SocketAddr, pub router: Router }

pub async fn build_app(cfg: AppConfig) -> anyhow::Result<App> {

    let bind = cfg.bind_addr.as_str();

    // Create application state
    let app_state = AppState::new(cfg.clone()).await?;

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // API routes
        .nest("/api", OpenApiRouter::from(create_api_router()))

        .layer(middleware::from_fn(metrics_and_logging_middleware))

        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())

        // Health endpoint
        .route("/health", get(|| async { "ok" }))

        // Layers
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)

        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));


    Ok(App { addr: SocketAddr::from_str(bind)?, router })
}

