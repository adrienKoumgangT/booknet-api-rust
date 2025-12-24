use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
    pub website: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEmbed {
    pub name: String,
}
