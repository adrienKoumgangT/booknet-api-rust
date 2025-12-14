use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::model::source_model::Source;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceResponse {
    pub name: String,
    pub website: String,
}


impl From<Source> for SourceResponse {
    fn from(source: Source) -> Self {
        Self {
            name: source.name,
            website: source.website,
        }
    }
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceCreateRequest {
    pub name: String,
    pub website: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceUpdateRequest {
    pub website: String,
}


