use axum::Router;
use crate::shared::state::AppState;
use crate::controller::publisher_controller::routes as publisher_routes;

pub fn routes() -> Router<AppState> {
    Router::new().merge(publisher_routes())
}
