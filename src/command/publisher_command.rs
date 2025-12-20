use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::shared::models::response::PaginationRequest;


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherGetCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherCreateCommand {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherUpdateCommand {
    pub name: String,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherDeleteCommand {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublisherListCommand {
    pub pagination: Option<PaginationRequest>,
}
