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

