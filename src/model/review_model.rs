use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub book_id: String,
    pub user_id: String,

    pub summary: String,
    pub content: String,
    pub score: f32,
    pub date: String,
    pub helpfulness: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rater {
    pub rating: f32,
    pub ts: DateTime<Utc>,
}
