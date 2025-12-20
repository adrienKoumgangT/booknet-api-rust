use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::book_model::BookEmbed;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    Reader,
}

impl UserRole {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Reader => "reader",
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

impl Default for UserRole {
    fn default() -> Self { Self::Reader }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub authors: Vec<String>,
    pub genres: Vec<String>,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub username: String,
    pub password_hash: String,

    pub name: String,
    pub image: Option<String>,

    pub role: UserRole,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preference: Option<UserPreference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shelf: Option<Vec<BookEmbed>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reviews: Option<Vec<String>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderNode {
    pub id: Option<String>,
    pub user_id: String,
    pub name: String,
}

impl From<&User> for ReaderNode {
    fn from(user: &User) -> Self {
        Self::new(user.id.clone().unwrap().to_hex(), user.name.clone())
    }
}

impl ReaderNode {
    pub fn new(user_id: String, name: String) -> Self {
        Self {
            id: None,
            user_id,
            name
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmbed {
    pub id: ObjectId,
    pub name: String,
    pub image: Option<String>,
}

impl From<&User> for UserEmbed {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.clone().unwrap(),
            name: user.name.clone(),
            image: user.image.clone(),
        }
    }
}


