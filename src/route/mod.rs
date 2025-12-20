use axum::Router;
use crate::shared::state::AppState;

mod genre_route;
mod language_route;
mod publisher_route;
mod source_route;



pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/genre", genre_route::routes())
        .nest("/language", language_route::routes())
        .nest("/publisher", publisher_route::routes())
        .nest("/source", source_route::routes())
}

