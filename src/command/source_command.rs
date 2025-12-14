use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use crate::shared::models::response::PaginationRequest;


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceGetCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceCreateCommand {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceUpdateCommand {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceDeleteCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceListCommand {
    pub pagination: Option<PaginationRequest>,
}

