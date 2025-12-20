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
pub struct BookAuthor {
    pub id: ObjectId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookGenre {
    pub id: ObjectId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookPublisher {
    pub id: ObjectId,
    pub name: String,
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
    pub published_date: Option<DateTime<Utc>>,
    pub format: BookFormat,

    pub images: Vec<BookImageSource>,
    pub preview: Vec<BookPreviewSource>,
    pub genres: Vec<BookGenre>,
    pub authors: Vec<BookAuthor>,
    pub publishers: Vec<BookPublisher>,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEmbed {
    pub book_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub image: Option<String>,
}

impl From<&Book> for BookEmbed {
    fn from(book: &Book) -> Self {
        Self {
            book_id: book.id.unwrap().clone(),
            title: book.title.clone(),
            description: book.description.clone(),
            image: book.images.first().map(|img| img.url.clone()),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookNode {
    pub book_id: String,
    pub title: String,
}

impl From<&Book> for BookNode {
    fn from(book: &Book) -> Self {
        Self {
            book_id: book.id.clone().unwrap().to_hex(),
            title: book.title.clone(),
        }
    }
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


