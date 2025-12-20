use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Metadata {
    Source {
        name: String,
        website: String,
    },
    Language {
        code: String,
        name: String,
    },
    Genre {
        name: String,
        description: String,
    },
    Publisher {
        name: String,
        website: String,
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDoc {
    #[serde(rename = "_id")]
    pub id: String, // "source:<name>" | "language:<code>" | "genre:<name>"

    pub key: String,  // "<name>" or "<code>" for easy queries

    #[serde(flatten)]
    pub meta: Metadata, // includes the "type" field because of #[serde(tag="type")]
}


impl Metadata {
    pub fn new_source(name: String, website: String) -> Self {
        Self::Source { name, website }
    }

    pub fn new_language(code: String, name: String) -> Self {
        Self::Language { code, name }
    }

    pub fn new_genre(name: String, description: String) -> Self {
        Self::Genre { name, description }
    }
    
    pub fn new_publisher(name: String, website: String) -> Self {
        Self::Publisher { name, website }
    }

    pub fn save_in_noe4j(&self) -> bool {
        match self {
            Metadata::Genre { .. } => true,
            _ => false,
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Metadata::Source { name, .. } => name,
            Metadata::Genre { name, .. } => name,
            Metadata::Language { code, .. } => code,
            Metadata::Publisher { name, .. } => name,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Metadata::Source { .. } => "source",
            Metadata::Language { .. } => "language",
            Metadata::Genre { .. } => "genre",
            Metadata::Publisher { .. } => "publisher",
        }
    }

    // If you want to avoid collisions in a single Mongo collection, prefer this:
    pub fn mongo_id(&self) -> String {
        format!("{}:{}", self.kind(), self.key())
    }

    pub fn to_doc(&self) -> MetadataDoc {
        MetadataDoc {
            id: self.mongo_id(),
            key: self.key().to_string(),
            meta: self.clone(),
        }
    }

    pub fn id_from(kind: &str, key: &str) -> String {
        format!("{kind}:{key}")
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataKey {
    Source { name: String },
    Language { code: String },
    Genre { name: String },
    Publisher { name: String },
}

impl MetadataKey {
    pub fn save_in_noe4j(&self) -> bool {
        match self {
            MetadataKey::Genre { .. } => true,
            _ => false,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            MetadataKey::Source { .. } => "source",
            MetadataKey::Language { .. } => "language",
            MetadataKey::Genre { .. } => "genre",
            MetadataKey::Publisher { .. } => "publisher",
        }
    }
    pub fn key(&self) -> &str {
        match self {
            MetadataKey::Source { name } => name,
            MetadataKey::Genre { name } => name,
            MetadataKey::Language { code } => code,
            MetadataKey::Publisher { name } => name,
        }
    }
    pub fn mongo_id(&self) -> String {
        format!("{}:{}", self.kind(), self.key())
    }
}
