use serde::{Deserialize, Serialize};
use crate::model::metadata_model::{Metadata, MetadataDoc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreEmbed {
    pub name: String,
}

impl From<&Genre> for GenreEmbed {
    fn from(genre: &Genre) -> Self {
        Self {
            name: genre.name.clone(),
        }
    }
}

impl From<&MetadataDoc> for GenreEmbed {
    fn from(doc: &MetadataDoc) -> Self {
        match &doc.meta {
            Metadata::Genre { name, .. } => Self { name: name.clone()},
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreNode {
    pub id: Option<String>,
    pub genre_id: String,
    pub name: String,
}

impl From<&Genre> for GenreNode {
    fn from(genre: &Genre) -> Self {
        Self {
            id: None,
            genre_id: genre.name.clone(),
            name: genre.name.clone(),
        }
    }
}

impl From<&MetadataDoc> for GenreNode {
    fn from(doc: &MetadataDoc) -> Self {
        match &doc.meta {
            Metadata::Genre { name, .. } => Self { id: None, genre_id: doc.id.clone(), name: name.clone()},
            _ => unreachable!(),
        }
    }
}
