use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::shared::models::response::PaginationRequest;


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreGetCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreCreateCommand {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreUpdateCommand {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreDeleteCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreListCommand {
    pub pagination: Option<PaginationRequest>,
}
