use axum::Router;
use crate::shared::state::AppState;
use crate::controller::genre_controller::routes as genre_controller_routes;
use crate::controller::language_controller::routes as language_controller_routes;
use crate::controller::source_controller::routes as source_controller_routes;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/genre", genre_controller_routes())
        .nest("/language", language_controller_routes())
        .nest("/source", source_controller_routes())
}
