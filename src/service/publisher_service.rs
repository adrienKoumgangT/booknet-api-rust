use anyhow::{Error, Result};
use async_trait::async_trait;


use crate::command::publisher_command::{
    PublisherCreateCommand, PublisherDeleteCommand, PublisherGetCommand, PublisherListCommand, PublisherUpdateCommand
};
use crate::dto::publisher_dto::PublisherResponse;
use crate::service::metadata_service::{MetadataService, MetadataServiceInterface};
use crate::shared::state::AppState;


#[async_trait]
pub trait PublisherServiceInterface {
    async fn get(&self, cmd: PublisherGetCommand) -> Result<Option<PublisherResponse>, Error>;
    async fn create(&self, cmd: PublisherCreateCommand) -> Result<PublisherResponse, Error>;
    async fn update(&self, cmd: PublisherUpdateCommand) -> Result<Option<PublisherResponse>, Error>;
    async fn delete(&self, cmd: PublisherDeleteCommand) -> Result<(), Error>;
    async fn list(&self, cmd: PublisherListCommand) -> Result<Vec<PublisherResponse>, Error>;
}


#[derive(Clone)]
pub struct PublisherService {
    metadata_service: MetadataService,
}

impl From<&AppState> for PublisherService {
    fn from(app_state: &AppState) -> Self {
        Self::new(app_state)
    }
}

impl PublisherService {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            metadata_service: MetadataService::from(app_state)
        }
    }
}

#[async_trait]
impl PublisherServiceInterface for PublisherService {
    async fn get(&self, cmd: PublisherGetCommand) -> Result<Option<PublisherResponse>, Error> {
        self.metadata_service.get_publisher(cmd).await
    }
    
    async fn create(&self, cmd: PublisherCreateCommand) -> Result<PublisherResponse, Error> {
        self.metadata_service.create_publisher(cmd).await
    }
    
    async fn update(&self, cmd: PublisherUpdateCommand) -> Result<Option<PublisherResponse>, Error> {
        self.metadata_service.update_publisher(cmd).await
    }
    
    async fn delete(&self, cmd: PublisherDeleteCommand) -> Result<(), Error> {
        self.metadata_service.delete_publisher(cmd).await
    }
    
    async fn list(&self, cmd: PublisherListCommand) -> Result<Vec<PublisherResponse>, Error> {
        self.metadata_service.list_publishers(cmd).await
    }
}
