use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::model::{publisher_model::Publisher, metadata_model::Metadata};


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherResponse {
    pub name: String,
    pub website: String,
}

impl From<Publisher> for PublisherResponse {
    fn from(publisher: Publisher) -> Self {
        Self { name: publisher.name, website: publisher.website }
    }
}

impl From<&Publisher> for PublisherResponse {
    fn from(publisher: &Publisher) -> Self {
        Self { name: publisher.name.clone(), website: publisher.website.clone() }
    }
}

impl From<Metadata> for PublisherResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Publisher { name, website } => Self { name, website },
            _ => panic!("Cannot convert Metadata to PublisherResponse"),
        }
    }
}

impl From<&Metadata> for PublisherResponse {
    fn from(metadata: &Metadata) -> Self {
        match metadata {
            Metadata::Publisher { name, website } => Self { name: name.clone(), website: website.clone() },
            _ => panic!("Cannot convert Metadata to PublisherResponse"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherCreateRequest {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherUpdateRequest {
    pub website: String,
}


