use axum::{Router, routing::{get}, extract::{Path, State}, Json, http::StatusCode};

use crate::command::publisher_command::{
    PublisherCreateCommand, PublisherDeleteCommand, PublisherGetCommand, PublisherListCommand, PublisherUpdateCommand
};
use crate::dto::publisher_dto::{PublisherCreateRequest, PublisherResponse, PublisherUpdateRequest};
use crate::service::publisher_service::{PublisherService, PublisherServiceInterface};
use crate::shared::state::AppState;


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_publishers).post(post_publisher))
        .route("/{publisher_id}", get(get_publisher).patch(put_publisher).delete(delete_publisher))
}


#[utoipa::path(
    get,
    path = "/api/services/publisher",
    responses(
        (status = StatusCode::OK, description = "List of publishers", body = Vec<PublisherResponse>),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Publisher"
)]
pub async fn list_publishers(State(state): State<AppState>) -> Result<Json<Vec<PublisherResponse>>, StatusCode> {
    let cmd = PublisherListCommand { pagination: None };
    let service = PublisherService::from(&state);
    let publishers = service.list(cmd).await;
    match publishers {
        Ok(publishers) => Ok(Json(publishers)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    post,
    path = "/api/services/publisher",
    responses(
        (status = StatusCode::CREATED, description = "Publisher created", body = PublisherResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Publisher"
)]
pub async fn post_publisher(State(state): State<AppState>, Json(publisher_create_request): Json<PublisherCreateRequest>) -> Result<Json<PublisherResponse>, StatusCode> {
    let cmd = PublisherCreateCommand { name: publisher_create_request.name, website: publisher_create_request.website };
    let service = PublisherService::from(&state);
    let publisher = service.create(cmd).await;
    match publisher {
        Ok(publisher) => Ok(Json(publisher)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    get,
    path = "/api/services/publisher/{publisher_id}",
    responses(
        (status = StatusCode::OK, description = "Publisher retrieved", body = PublisherResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Publisher not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Publisher"
)]
pub async fn get_publisher(
    Path(publisher_id): Path<String>,
    State(state): State<AppState>
) -> Result<Json<PublisherResponse>, StatusCode> {
    let cmd = PublisherGetCommand { id: publisher_id };
    let service = PublisherService::from(&state);
    let publisher = service.get(cmd).await;
    match publisher {
        Ok(publisher) => {
            match publisher {
                Some(publisher) => Ok(Json(publisher)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    put,
    path = "/api/services/publisher/{publisher_id}",
    responses(
        (status = StatusCode::OK, description = "Publisher updated", body = PublisherResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Publisher not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Publisher"
)]
pub async fn put_publisher(
    Path(publisher_id): Path<String>,
    State(state): State<AppState>,
    Json(publisher_update_request): Json<PublisherUpdateRequest>
) -> Result<Json<PublisherResponse>, StatusCode> {
    let cmd = PublisherUpdateCommand { name: publisher_id, website: publisher_update_request.website };
    let service = PublisherService::from(&state);
    let publisher = service.update(cmd).await;
    match publisher {
        Ok(publisher) => {
            match publisher {
                Some(publisher) => Ok(Json(publisher)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    delete,
    path = "/api/services/publisher/{publisher_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Publisher deleted"),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Publisher not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Publisher"
)]
pub async fn delete_publisher(
    Path(publisher_id): Path<String>,
    State(state): State<AppState>
) -> Result<(), StatusCode> {
    let cmd = PublisherDeleteCommand { id: publisher_id };
    let service = PublisherService::from(&state);
    let result = service.delete(cmd).await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

