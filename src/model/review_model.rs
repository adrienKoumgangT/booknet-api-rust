use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub book_id: ObjectId,
    pub user_id: ObjectId,

    pub summary: Option<String>,
    pub content: String,
    pub score: f32,
    pub date_added: Option<DateTime<Utc>>,
    pub helpfulness: Option<i32>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaterRelationShip {
    pub rating: f32,
    pub ts: DateTime<Utc>,
}
