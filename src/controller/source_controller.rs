use axum::{Router, routing::{get}, extract::{Path, State}, Json, http::StatusCode};
use crate::shared::state::AppState;

use crate::command::source_command::{
    SourceCreateCommand,
    SourceDeleteCommand,
    SourceGetCommand,
    SourceListCommand,
    SourceUpdateCommand
};
use crate::dto::source_dto::{SourceCreateRequest, SourceResponse, SourceUpdateRequest};
use crate::service::source_service::{SourceService, SourceServiceInterface};


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_sources).post(post_source))
        .route("/{source_id}", get(get_source).put(put_source).delete(delete_source))
}


#[utoipa::path(
    get,
    path = "/api/source",
    responses(
        (status = StatusCode::OK, description = "List of sources", body = Vec<SourceResponse>),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn get_sources(State(state): State<AppState>) -> Result<Json<Vec<SourceResponse>>, StatusCode> {
    let service_list_command = SourceListCommand { pagination: None };
    let source_service = SourceService::from(&state);
    let sources = source_service.list(service_list_command).await;
    match sources {
        Ok(sources) => Ok(Json(sources)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    post,
    path = "/api/source",
    responses(
        (status = StatusCode::CREATED, description = "Source created", body = SourceResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Source"
)]
pub async fn post_source(State(state): State<AppState>, Json(source_create_request): Json<SourceCreateRequest>) -> Result<Json<SourceResponse>, StatusCode> {
    let source_create_command = SourceCreateCommand { name: source_create_request.name, website: source_create_request.website };
    let source_service = SourceService::from(&state);
    let source = source_service.create(source_create_command).await;
    match source {
        Ok(source) => Ok(Json(source)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    get,
    path = "/api/source/{source_id}",
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
    let source_get_command = SourceGetCommand { id: source_id };
    let source_service = SourceService::from(&state);
    let source = source_service.get(source_get_command).await;
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
    path = "/api/source/{source_id}",
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
    let source_update_command = SourceUpdateCommand { name: source_id, website: source_update_request.website };
    let source_service = SourceService::from(&state);
    let source = source_service.update(source_update_command).await;
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
    path = "/api/source/{source_id}",
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
    let source_delete_command = SourceDeleteCommand { id: source_id };
    let source_service = SourceService::from(&state);
    let result = source_service.delete(source_delete_command).await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
