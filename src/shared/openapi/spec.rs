use utoipa::{OpenApi};

use crate::controller::{source_controller};
use crate::dto::{source_dto};

#[derive(OpenApi)]
#[openapi(
    info(version = "1.0.0", title = "Book Net API", description = "Book Net API description"),
    tags(
        (name = "Source", description = "Source API endpoints"),
        (name = "User", description = "User API endpoints"),
    ),
    paths(
        source_controller::get_sources, source_controller::post_source,
        source_controller::get_source, source_controller::put_source, source_controller::delete_source
    ),
    components(
        schemas(
            source_dto::SourceResponse, source_dto::SourceCreateRequest, source_dto::SourceUpdateRequest,
        )
    )
)]
pub struct ApiDoc;
