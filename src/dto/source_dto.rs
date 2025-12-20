use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::metadata_model::Metadata;
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

impl From<Metadata> for SourceResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Source { name, website } => Self { name, website },
            _ => panic!("Cannot convert Metadata to SourceResponse"),
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