use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookImageSource {
    pub url: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookPreviewSource {
    pub url: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookFormat {
    Paperback,
    Hardcover,
    EBook,
    Audiobook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub isbn: String,
    pub isbn13: String,

    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub num_pages: Option<i32>,
    pub published_date: DateTime<Utc>,
    pub format: BookFormat,

    pub images: Vec<BookImageSource>,
    pub preview: Vec<BookPreviewSource>,
    pub genres: Vec<String>,
    pub authors: Vec<String>,
    pub publishers: Vec<String>,
    pub languages: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookNode {
    pub book_id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BookReadStatus {
    Read,
    Unread,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddedToShelf {
    pub status: String,
    pub ts: DateTime<Utc>,
}


