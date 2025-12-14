use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Reader,
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
    pub reviews: Option<Vec<String>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderNode {
    pub user_id: String,
    pub username: String,
    pub name: String,
}

impl ReaderNode {
    pub fn new(user: &User) -> Self {
        Self {
            user_id: user.id.clone().unwrap().to_hex(),
            username: user.username.clone(),
            name: user.name.clone(),
        }
    }
}


