use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// A generic paginated request structure.
#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PaginationRequest {
    #[param(example = 1)]
    pub page: Option<u32>,
    #[param(example = 10)]
    pub page_size: Option<u32>,
    #[param(example = "name")]
    pub search: Option<String>,
}

/// A generic paginated response structure.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}


/// API Responses
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}


impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}
