use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::user_model::UserEmbed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub book_id: ObjectId,
    pub user: UserEmbed,

    pub content: String,
    pub score: f32,
    pub date_added: Option<DateTime<Utc>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaterRelationShip {
    pub rating: f32,
    pub ts: i64,
}
