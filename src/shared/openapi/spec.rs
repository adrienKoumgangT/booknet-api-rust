use utoipa::{OpenApi};

use crate::controller::{language_controller, source_controller};
use crate::dto::{language_dto, source_dto};

#[derive(OpenApi)]
#[openapi(
    info(version = "1.0.0", title = "Book Net API", description = "Book Net API description"),
    tags(
        (name = "Language", description = "Language API endpoints"),
        (name = "Source", description = "Source API endpoints"),
        (name = "User", description = "User API endpoints"),
    ),
    paths(
        source_controller::get_sources, source_controller::post_source,
        source_controller::get_source, source_controller::put_source, source_controller::delete_source,

        language_controller::get_languages, language_controller::post_language,
        language_controller::get_language, language_controller::put_language, language_controller::delete_language,
    ),
    components(
        schemas(
            source_dto::SourceResponse, source_dto::SourceCreateRequest, source_dto::SourceUpdateRequest,
            language_dto::LanguageResponse, language_dto::LanguageCreateRequest, language_dto::LanguageUpdateRequest,
        )
    )
)]
pub struct ApiDoc;
