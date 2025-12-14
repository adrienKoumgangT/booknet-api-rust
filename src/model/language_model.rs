use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    #[serde(rename = "_id")]
    pub code: String,
    pub name: String,
}

impl Language {
    pub fn new(code: String, name: String) -> Self {
        Self {
            code,
            name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEmbed {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageNode {
    pub code: String,
    pub name: String,
}

