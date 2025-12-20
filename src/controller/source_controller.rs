use axum::{Router, routing::{get}, extract::{Path, State}, Json, http::StatusCode};

use crate::command::source_command::{
    SourceCreateCommand,
    SourceDeleteCommand,
    SourceGetCommand,
    SourceListCommand,
    SourceUpdateCommand
};
use crate::dto::source_dto::{SourceCreateRequest, SourceResponse, SourceUpdateRequest};
use crate::service::source_service::{SourceService, SourceServiceInterface};
use crate::shared::state::AppState;


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_sources).post(post_source))
        .route("/{source_id}", get(get_source).put(put_source).delete(delete_source))
}


#[utoipa::path(
    get,
    path = "/api/services/source",
    responses(
        (status = StatusCode::OK, description = "List of sources", body = Vec<SourceResponse>),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn get_sources(State(state): State<AppState>) -> Result<Json<Vec<SourceResponse>>, StatusCode> {
    let cmd = SourceListCommand { pagination: None };
    let service = SourceService::from(&state);
    let sources = service.list(cmd).await;
    match sources {
        Ok(sources) => Ok(Json(sources)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    post,
    path = "/api/services/source",
    responses(
        (status = StatusCode::CREATED, description = "Source created", body = SourceResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn post_source(State(state): State<AppState>, Json(source_create_request): Json<SourceCreateRequest>) -> Result<Json<SourceResponse>, StatusCode> {
    let cmd = SourceCreateCommand { name: source_create_request.name, website: source_create_request.website };
    let service = SourceService::from(&state);
    let source = service.create(cmd).await;
    match source {
        Ok(source) => Ok(Json(source)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    get,
    path = "/api/services/source/{source_id}",
    responses(
        (status = StatusCode::OK, description = "Source retrieved", body = SourceResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Source not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn get_source(
    Path(source_id): Path<String>,
    State(state): State<AppState>
) -> Result<Json<SourceResponse>, StatusCode> {
    let cmd = SourceGetCommand { id: source_id };
    let service = SourceService::from(&state);
    let source = service.get(cmd).await;
    match source {
        Ok(source) => {
            match source {
                Some(source) => Ok(Json(source)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    put,
    path = "/api/services/source/{source_id}",
    responses(
        (status = StatusCode::OK, description = "Source updated", body = SourceResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Source not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn put_source(
    Path(source_id): Path<String>,
    State(state): State<AppState>,
    Json(source_update_request): Json<SourceUpdateRequest>
) -> Result<Json<SourceResponse>, StatusCode> {
    let cmd = SourceUpdateCommand { name: source_id, website: source_update_request.website };
    let service = SourceService::from(&state);
    let source = service.update(cmd).await;
    match source {
        Ok(source) => {
            match source {
                Some(source) => Ok(Json(source)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    delete,
    path = "/api/services/source/{source_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Source deleted"),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Source not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn delete_source(
    Path(source_id): Path<String>,
    State(state): State<AppState>
) -> Result<(), StatusCode> {
    let cmd = SourceDeleteCommand { id: source_id };
    let service = SourceService::from(&state);
    let result = service.delete(cmd).await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
