use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use crate::model::metadata_model::{Source, Language, Genre, Metadata};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceResponse {
    pub name: String,
    pub website: String,
}


impl From<Source> for SourceResponse {
    fn from(source: Source) -> Self {
        Self {
            name: source.name,
            website: source.website,
        }
    }
}

impl From<Metadata> for SourceResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Source { name, website } => Self { name, website },
            _ => panic!("Cannot convert Metadata to SourceResponse"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceCreateRequest {
    pub name: String,
    pub website: String,
}


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceUpdateRequest {
    pub website: String,
}



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



#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreResponse {
    pub name: String,
    pub description: String,
}

impl From<Genre> for GenreResponse {
    fn from(genre: Genre) -> Self {
        Self {
            name: genre.name,
            description: genre.description,
        }
    }
}

impl From<Metadata> for GenreResponse {
    fn from(metadata: Metadata) -> Self {
        match metadata {
            Metadata::Genre { name, description } => Self { name, description },
            _ => panic!("Cannot convert Metadata to GenreResponse"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreCreateRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenreUpdateRequest {
    pub description: String,
}
