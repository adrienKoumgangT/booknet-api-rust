use axum::Router;
use crate::shared::state::AppState;
use crate::controller::language_controller::routes as language_routes;

pub fn routes() -> Router<AppState> {
    Router::new().merge(language_routes())
}
