use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::model::genre_model::Genre;
use crate::model::metadata_model::{Metadata};


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreResponse {
    pub name: String,
    pub description: String,
}

impl From<Genre> for GenreResponse {
    fn from(genre: Genre) -> Self {
        Self {
            name: genre.name,
            description: genre.description,
        }
    }
}

impl From<&Genre> for GenreResponse {
    fn from(genre: &Genre) -> Self {
        Self {
            name: genre.name.clone(),
            description: genre.description.clone(),
        }
    }
}

impl From<Metadata> for GenreResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Genre { name, description } => Self { name, description },
            _ => panic!("Cannot convert Metadata to GenreResponse"),
        }
    }
}

impl From<&Metadata> for GenreResponse {
    fn from(metadata: &Metadata) -> Self {
        match &metadata {
            Metadata::Genre { name, description } => Self { name: name.clone(), description: description.clone() },
            _ => panic!("Cannot convert Metadata to GenreResponse"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreCreateRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreUpdateRequest {
    pub description: String,
}
