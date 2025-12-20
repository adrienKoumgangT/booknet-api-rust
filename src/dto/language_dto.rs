use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::language_model::Language;
use crate::model::metadata_model::Metadata;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageResponse {
    pub code: String,
    pub name: String,
}

impl From<Language> for LanguageResponse {
    fn from(language: Language) -> Self {
        Self {
            code: language.code,
            name: language.name,
        }
    }
}

impl From<Metadata> for LanguageResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Language { code, name } => Self { code, name },
            _ => panic!("Cannot convert Metadata to LanguageResponse"),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageCreateRequest {
    pub code: String,
    pub name: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageUpdateRequest {
    pub name: String,
}
