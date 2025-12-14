use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    #[serde(rename = "_id")]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreEmbed {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreNode {
    pub name: String,
}

