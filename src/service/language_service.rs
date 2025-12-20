use anyhow::{Error, Result};
use async_trait::async_trait;

use crate::command::language_command::{
    LanguageCreateCommand, LanguageDeleteCommand, LanguageGetCommand, LanguageListCommand, LanguageUpdateCommand,
};
use crate::dto::language_dto::LanguageResponse;
use crate::service::metadata_service::{MetadataService, MetadataServiceInterface};
use crate::shared::state::AppState;


#[async_trait]
pub trait LanguageServiceInterface {
    async fn get(&self, cmd: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn create(&self, cmd: LanguageCreateCommand) -> Result<LanguageResponse, Error>;
    async fn update(&self, cmd: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn delete(&self, cmd: LanguageDeleteCommand) -> Result<(), Error>;
    async fn list(&self, cmd: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error>;
}


#[derive(Clone)]
pub struct LanguageService {
    metadata_service: MetadataService,
}

impl From<&AppState> for LanguageService {
    fn from(app_state: &AppState) -> Self {
        Self::new(app_state)
    }
}

impl LanguageService {
    fn new(app_state: &AppState) -> Self {
        Self {
            metadata_service: MetadataService::from(app_state)
        }
    }
}

#[async_trait]
impl LanguageServiceInterface for LanguageService {
    async fn get(&self, cmd: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error> {
        self.metadata_service.get_language(cmd).await
    }
    
    async fn create(&self, cmd: LanguageCreateCommand) -> Result<LanguageResponse, Error> {
        self.metadata_service.create_language(cmd).await
    }
    
    async fn update(&self, cmd: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error> {
        self.metadata_service.update_language(cmd).await
    }
    
    async fn delete(&self, cmd: LanguageDeleteCommand) -> Result<(), Error> {
        self.metadata_service.delete_language(cmd).await
    }
    
    async fn list(&self, cmd: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error> {
        self.metadata_service.list_languages(cmd).await
    }
}
