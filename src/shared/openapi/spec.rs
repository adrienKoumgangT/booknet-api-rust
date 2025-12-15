use utoipa::{OpenApi};

use crate::controller::{genre_controller, language_controller, source_controller};
use crate::dto::{metadata_dto};

#[derive(OpenApi)]
#[openapi(
    info(version = "1.0.0", title = "Book Net API", description = "Book Net API description"),
    tags(
        (name = "Genre", description = "Genre API endpoints"),
        (name = "Language", description = "Language API endpoints"),
        (name = "Source", description = "Source API endpoints"),
        (name = "User", description = "User API endpoints"),
    ),
    paths(

        genre_controller::get_genres, genre_controller::post_genre,
        genre_controller::get_genre, genre_controller::put_genre, genre_controller::delete_genre,

        language_controller::get_languages, language_controller::post_language,
        language_controller::get_language, language_controller::put_language, language_controller::delete_language,
    
        source_controller::get_sources, source_controller::post_source,
        source_controller::get_source, source_controller::put_source, source_controller::delete_source,
    ),
    components(
        schemas(
            metadata_dto::GenreResponse, metadata_dto::GenreCreateRequest, metadata_dto::GenreUpdateRequest,
            metadata_dto::LanguageResponse, metadata_dto::LanguageCreateRequest, metadata_dto::LanguageUpdateRequest,
            metadata_dto::SourceResponse, metadata_dto::SourceCreateRequest, metadata_dto::SourceUpdateRequest,
        )
    )
)]
pub struct ApiDoc;
