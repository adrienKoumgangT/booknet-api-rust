use anyhow::{Error, Result};
use async_trait::async_trait;

use crate::command::genre_command::{
    GenreCreateCommand, GenreDeleteCommand, GenreGetCommand, GenreListCommand, GenreUpdateCommand,
};
use crate::dto::genre_dto::GenreResponse;
use crate::service::metadata_service::{MetadataService, MetadataServiceInterface};
use crate::shared::state::AppState;


#[async_trait]
pub trait GenreServiceInterface {
    async fn get(&self, cmd: GenreGetCommand) -> Result<Option<GenreResponse>, Error>;
    async fn create(&self, cmd: GenreCreateCommand) -> Result<GenreResponse, Error>;
    async fn update(&self, cmd: GenreUpdateCommand) -> Result<Option<GenreResponse>, Error>;
    async fn delete(&self, cmd: GenreDeleteCommand) -> Result<(), Error>;
    async fn list(&self, cmd: GenreListCommand) -> Result<Vec<GenreResponse>, Error>;
}


#[derive(Clone)]
pub struct GenreService {
    metadata_service: MetadataService,
}

impl From<&AppState> for GenreService {
    fn from(app_state: &AppState) -> Self {
        Self::new(app_state)
    }
}

impl GenreService {
    fn new(app_state: &AppState) -> Self {
        Self {
            metadata_service: MetadataService::from(app_state)
        }
    }
}

#[async_trait]
impl GenreServiceInterface for GenreService {
    async fn get(&self, cmd: GenreGetCommand) -> Result<Option<GenreResponse>, Error> {
        self.metadata_service.get_genre(cmd).await
    }
    
    async fn create(&self, cmd: GenreCreateCommand) -> Result<GenreResponse, Error> {
        self.metadata_service.create_genre(cmd).await
    }

    async fn update(&self, cmd: GenreUpdateCommand) -> Result<Option<GenreResponse>, Error> {
        self.metadata_service.update_genre(cmd).await
    }

    async fn delete(&self, cmd: GenreDeleteCommand) -> Result<(), Error> {
        self.metadata_service.delete_genre(cmd).await
    }

    async fn list(&self, cmd: GenreListCommand) -> Result<Vec<GenreResponse>, Error> {
        self.metadata_service.list_genres(cmd).await
    }
}
