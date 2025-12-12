use utoipa::{OpenApi};



#[derive(OpenApi)]
#[openapi(
    info(version = "1.0.0", title = "E-Commerce API", description = "E-Commerce API description"),
    tags(
        (name = "User", description = "User API endpoints"),
    ),
    paths(
        
    ),
    components(
        schemas(
        )
    )
)]
pub struct ApiDoc;
