use anyhow::{Error, Result};
use async_trait::async_trait;

use crate::command::source_command::{
    SourceCreateCommand, SourceDeleteCommand, SourceGetCommand, SourceListCommand, SourceUpdateCommand
};
use crate::dto::source_dto::SourceResponse;
use crate::service::metadata_service::{MetadataService, MetadataServiceInterface};
use crate::shared::state::AppState;



#[async_trait]
pub trait SourceServiceInterface {
    async fn get(&self, cmd: SourceGetCommand) -> Result<Option<SourceResponse>, Error>;
    async fn create(&self, cmd: SourceCreateCommand) -> Result<SourceResponse, Error>;
    async fn update(&self, cmd: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error>;
    async fn delete(&self, cmd: SourceDeleteCommand) -> Result<(), Error>;
    async fn list(&self, cmd: SourceListCommand) -> Result<Vec<SourceResponse>, Error>;
}


#[derive(Clone)]
pub struct SourceService {
    metadata_service: MetadataService,
}


impl From<&AppState> for SourceService {
    fn from(app_state: &AppState) -> Self {
        Self::new(app_state)
    }
}

impl SourceService {
    fn new(app_state: &AppState) -> Self {
        Self {
            metadata_service: MetadataService::from(app_state)
        }
    }
}

#[async_trait]
impl SourceServiceInterface for SourceService {
    async fn get(&self, cmd: SourceGetCommand) -> Result<Option<SourceResponse>, Error> {
        self.metadata_service.get_source(cmd).await
    }

    async fn create(&self, cmd: SourceCreateCommand) -> Result<SourceResponse, Error> {
        self.metadata_service.create_source(cmd).await
    }

    async fn update(&self, cmd: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error> {
        self.metadata_service.update_source(cmd).await
    }

    async fn delete(&self, cmd: SourceDeleteCommand) -> Result<(), Error> {
        self.metadata_service.delete_source(cmd).await
    }

    async fn list(&self, cmd: SourceListCommand) -> Result<Vec<SourceResponse>, Error> {
        self.metadata_service.list_sources(cmd).await
    }
}
