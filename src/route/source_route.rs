use axum::Router;
use crate::shared::state::AppState;
use crate::controller::source_controller::routes as source_routes;

pub fn routes() -> Router<AppState> {
    Router::new().merge(source_routes())
}
