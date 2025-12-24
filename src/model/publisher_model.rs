use serde::{Deserialize, Serialize};
use crate::model::metadata_model::{Metadata, MetadataDoc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherEmbed {
    pub name: String,
}

impl From<&Publisher> for PublisherEmbed {
    fn from(publisher: &Publisher) -> Self {
        Self {
            name: publisher.name.clone(),
        }
    }
}

impl From<&MetadataDoc> for PublisherEmbed {
    fn from(doc: &MetadataDoc) -> Self {
        match &doc.meta {
            Metadata::Publisher { name, website } => Self { name: name.clone() },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherNode {
    pub id: Option<String>,
    pub publisher_id: String,
    pub name: String,
}

impl From<&Publisher> for PublisherNode {
    fn from(publisher: &Publisher) -> Self {
        Self {
            id: None,
            publisher_id: publisher.name.clone(),
            name: publisher.name.clone(),
        }
    }
}

impl From<&MetadataDoc> for PublisherNode {
    fn from(doc: &MetadataDoc) -> Self {
        match &doc.meta {
            Metadata::Publisher { name, website } => Self { id: None, publisher_id: doc.id.clone(), name: name.clone() },
            _ => unreachable!(),
        }
    }
}


