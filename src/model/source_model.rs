use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceEnum {
    Goodreads,
    Amazon,
    GoogleBooks,
}

impl SourceEnum {
    pub fn as_str(&self) -> &str {
        match self {
            SourceEnum::Goodreads => "goodreads",
            SourceEnum::Amazon => "amazon",
            SourceEnum::GoogleBooks => "google_books",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "_id")]
    pub name: String,
    pub website: String,
}

impl Source {
    pub fn new(name: String, website: String) -> Self {
        Self {
            name,
            website,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSource {
    pub name: String,
}


