use axum::{Router, routing::{get}, extract::{Path, State}, Json, http::StatusCode};
use crate::shared::state::AppState;

use crate::command::language_command::{LanguageCreateCommand, LanguageDeleteCommand, LanguageGetCommand, LanguageListCommand, LanguageUpdateCommand};
use crate::dto::language_dto::{LanguageCreateRequest, LanguageResponse, LanguageUpdateRequest};
use crate::service::language_service::{LanguageService, LanguageServiceInterface};


pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_languages).post(post_language))
        .route("/{language_id}", get(get_language).put(put_language).delete(delete_language))
}


#[utoipa::path(
    get,
    path = "/api/language",
    responses(
        (status = StatusCode::OK, description = "List of languages", body = Vec<LanguageResponse>),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Language"
)]
pub async fn get_languages(State(state): State<AppState>) -> Result<Json<Vec<LanguageResponse>>, StatusCode> {
    let service_list_command = LanguageListCommand { pagination: None };
    let language_service = LanguageService::from(&state);
    let languages = language_service.list(service_list_command).await;
    match languages {
        Ok(languages) => Ok(Json(languages)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    post,
    path = "/api/language",
    responses(
        (status = StatusCode::CREATED, description = "Language created", body = LanguageResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Language"
)]
pub async fn post_language(State(state): State<AppState>, Json(language_create_request): Json<LanguageCreateRequest>) -> Result<Json<LanguageResponse>, StatusCode> {
    let language_create_command = LanguageCreateCommand { code: language_create_request.code, name: language_create_request.name };
    let language_service = LanguageService::from(&state);
    let language = language_service.create(language_create_command).await;
    match language {
        Ok(language) => Ok(Json(language)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    get,
    path = "/api/language/{language_id}",
    responses(
        (status = StatusCode::OK, description = "Language retrieved", body = LanguageResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Language not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Language"
)]
pub async fn get_language(
    Path(language_id): Path<String>,
    State(state): State<AppState>
) -> Result<Json<LanguageResponse>, StatusCode> {
    let language_get_command = LanguageGetCommand { id: language_id };
    let language_service = LanguageService::from(&state);
    let language = language_service.get(language_get_command).await;
    match language {
        Ok(language) => {
            match language {
                Some(language) => Ok(Json(language)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    put,
    path = "/api/language/{language_id}",
    responses(
        (status = StatusCode::OK, description = "Language updated", body = LanguageResponse),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Language not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Language"
)]
pub async fn put_language(
    Path(language_id): Path<String>,
    State(state): State<AppState>,
    Json(language_update_request): Json<LanguageUpdateRequest>
) -> Result<Json<LanguageResponse>, StatusCode> {
    let language_update_command = LanguageUpdateCommand { code: language_id, name: language_update_request.name };
    let language_service = LanguageService::from(&state);
    let language = language_service.update(language_update_command).await;
    match language {
        Ok(language) => {
            match language {
                Some(language) => Ok(Json(language)),
                None => Err(StatusCode::NOT_FOUND)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}


#[utoipa::path(
    delete,
    path = "/api/language/{language_id}",
    responses(
        (status = StatusCode::NO_CONTENT, description = "Language deleted"),
        (status = StatusCode::BAD_REQUEST, description = "Bad request"),
        (status = StatusCode::NOT_FOUND, description = "Language not found"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal server error")
    ),
    tag = "Language"
)]
pub async fn delete_language(
    Path(language_id): Path<String>,
    State(state): State<AppState>
) -> Result<(), StatusCode> {
    let language_delete_command = LanguageDeleteCommand { id: language_id };
    let language_service = LanguageService::from(&state);
    let result = language_service.delete(language_delete_command).await;
    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
