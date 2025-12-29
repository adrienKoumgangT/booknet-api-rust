use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::book_model::BookEmbed;
use crate::model::external_id_model::ExternalId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    pub name: String,
    pub image_url: String,
    pub description: String,
    
    pub books: Vec<BookEmbed>,
    
    pub external_id: Option<ExternalId>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorEmbed {
    pub id: ObjectId,
    pub name: String,
    pub image_url: String,
}


impl From<&Author> for AuthorEmbed {
    fn from(author: &Author) -> Self {
        Self {
            id: author.id.clone().unwrap(),
            name: author.name.clone(),
            image_url: author.image_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorNode {
    pub id: Option<String>,
    pub author_id: String,
    pub name: String,
}

impl From<&Author> for AuthorNode {
    fn from(author: &Author) -> Self {
        Self {
            id: None,
            author_id: author.id.unwrap().to_hex(),
            name: author.name.clone(),
        }
    }
}
