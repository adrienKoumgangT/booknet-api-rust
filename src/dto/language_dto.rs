use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::model::language_model::Language;

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


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageCreateRequest {
    pub code: String,
    pub name: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LanguageUpdateRequest {
    pub name: String,
}

