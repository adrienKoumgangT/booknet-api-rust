use axum::{Router, routing::{get}, extract::{Path, State}, Json, http::StatusCode};

use crate::command::genre_command::{GenreCreateCommand, GenreDeleteCommand, GenreGetCommand, GenreListCommand, GenreUpdateCommand};
use crate::dto::genre_dto::{GenreCreateRequest, GenreResponse, GenreUpdateRequest};
use crate::service::genre_service::{GenreService, GenreServiceInterface};
use crate::shared::state::AppState;


pub fn routes() -> Router<AppState> {
    Router::new()
    .route("/", get(get_genres).post(post_genre))
    .route("/{genre_id}", get(get_genre).put(put_genre).delete(delete_genre))
}



#[utoipa::path(
    get,
    path = "/api/services/genre",
    responses(
        (status = StatusCode::OK, description = "List of genres", body = Vec<GenreResponse>),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Genre"
)]
pub async fn get_genres(State(state): State<AppState>) -> Result<Json<Vec<GenreResponse>>, StatusCode> {
    let cmd = GenreListCommand { pagination: None };
    let service = GenreService::from(&state);
    let genres = service.list(cmd).await;
    match genres {
        Ok(genres) => Ok(Json(genres)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    post,
    path = "/api/services/genre",
    responses(
        (status = StatusCode::CREATED, description = "Genre created", body = GenreResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Genre"
)]
pub async fn post_genre(State(state): State<AppState>, Json(request): Json<GenreCreateRequest>) -> Result<Json<GenreResponse>, StatusCode> {
    let cmd = GenreCreateCommand { name: request.name, description: request.description };
    let service = GenreService::from(&state);
    let genre = service.create(cmd).await;
    match genre {
        Ok(genre) => Ok(Json(genre)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
get,
    path = "/api/services/genre/{genre_id}",
    responses(
        (status = StatusCode::OK, description = "Genre retrieved", body = GenreResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Genre not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Genre"
)]
pub async fn get_genre(
    Path(genre_id): Path<String>,
    State(state): State<AppState>
) -> Result<Json<GenreResponse>, StatusCode> {
    let cmd = GenreGetCommand { id: genre_id };
    let service = GenreService::from(&state);
    let genre = service.get(cmd).await;
    match genre {
        Ok(genre) => {
            match genre {
                Some(genre) => Ok(Json(genre)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    put,
    path = "/api/services/genre/{genre_id}",
    responses(
        (status = StatusCode::OK, description = "Genre updated", body = GenreResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Genre not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Genre"
)]
pub async fn put_genre(
    Path(genre_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<GenreUpdateRequest>
) -> Result<Json<GenreResponse>, StatusCode> {
    let cmd = GenreUpdateCommand { name: genre_id, description: request.description };
    let service = GenreService::from(&state);
    let genre = service.update(cmd).await;
    match genre {
        Ok(genre) => {
            match genre {
                Some(genre) => Ok(Json(genre)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    delete,
    path = "/api/services/genre/{genre_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Genre deleted"),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Genre not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Genre"
)]
pub async fn delete_genre(
    Path(genre_id): Path<String>,
    State(state): State<AppState>
) -> Result<(), StatusCode> {
    let cmd = GenreDeleteCommand { id: genre_id };
    let service = GenreService::from(&state);
    let result = service.delete(cmd).await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
