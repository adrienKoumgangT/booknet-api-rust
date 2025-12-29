use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalId {
    pub good_reads: Option<String>,
    pub amazon: Option<String>,
    pub google_books: Option<String>,
    pub kaggle: Option<String>,
}


impl ExternalId {
    fn from_good_reads(external_id: &str) -> Self {
        Self {
            good_reads: Some(external_id.to_string()),
            amazon: None,
            google_books: None,
            kaggle: None,
        }
    }
    
    fn from_amazon(external_id: &str) -> Self {
        Self {
            good_reads: None,
            amazon: Some(external_id.to_string()),
            google_books: None,
            kaggle: None,
        }
    }
    
    fn from_google_books(external_id: &str) -> Self {
        Self {
            good_reads: None,
            amazon: None,
            google_books: Some(external_id.to_string()),
            kaggle: None,
        }
    }
    
    fn from_kaggle(external_id: &str) -> Self {
        Self {
            good_reads: None,
            amazon: None,
            google_books: None,
            kaggle: Some(external_id.to_string()),
        }
    }
}
