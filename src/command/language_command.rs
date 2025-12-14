use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use crate::shared::models::response::PaginationRequest;


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageGetCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageCreateCommand {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageUpdateCommand {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageDeleteCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageListCommand {
    pub pagination: Option<PaginationRequest>,
}

