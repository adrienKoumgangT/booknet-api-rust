use axum::Router;
use crate::shared::state::AppState;
use crate::controller::genre_controller::routes as genre_routes;

pub fn routes() -> Router<AppState> {
    Router::new().merge(genre_routes())
}
