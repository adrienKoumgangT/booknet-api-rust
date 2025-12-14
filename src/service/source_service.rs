use anyhow::{Error, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;

use crate::command::source_command::{
    SourceCreateCommand, 
    SourceDeleteCommand, 
    SourceGetCommand, 
    SourceListCommand, 
    SourceUpdateCommand
};
use crate::dto::source_dto::{SourceResponse};
use crate::model::source_model::{Source};
use crate::repository::source_repository::{SourceRepository, SourceRepositoryInterface};
use crate::shared::database::redis::{delete_key, get_key, set_key};
use crate::shared::state::AppState;


#[async_trait]
pub trait SourceServiceInterface {
    async fn get(&self, source_get_command: SourceGetCommand) -> Result<Option<SourceResponse>, Error>;
    async fn create(&self, source_create_command: SourceCreateCommand) -> Result<SourceResponse, Error>;
    async fn update(&self, source_update_command: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error>;
    async fn delete(&self, source_delete_command: SourceDeleteCommand) -> Result<(), Error>;
    async fn list(&self, _: SourceListCommand) -> Result<Vec<SourceResponse>, Error>;
}


#[derive(Clone)]
pub struct SourceService {
    source_repo: SourceRepository,
    redis_pool: Option<Pool<RedisConnectionManager>>,
    space_name: Option<String>,
}

impl From<&AppState> for SourceService {
    fn from(app_state: &AppState) -> Self {
        let database = app_state.mongo_client.database("booknet").clone();
        let space_name = app_state
            .config
            .database
            .redis
            .as_ref()
            .and_then(|r| r.app_space_name.as_deref())
            .unwrap_or("booknet")
            .to_string();

        Self::new(SourceRepository::new(database), Option::from(app_state.redis_pool.clone()), Option::from(space_name))
    }
}

impl SourceService {
    pub fn new(source_repo: SourceRepository, redis_pool: Option<Pool<RedisConnectionManager>>, space_name: Option<String>) -> Self {
        Self {
            source_repo,
            redis_pool,
            space_name
        }
    }

    fn redis_prefix(&self) -> &str {
        self.space_name
            .as_deref()
            .unwrap_or("")
    }

    fn redis_prefix_colon(&self) -> String {
        self.space_name
            .as_deref()
            .map(|s| format!("{s}:"))
            .unwrap_or_default()
    }

    pub fn redis_key_single_ttl(&self) -> Option<u64> {
        Some(60 * 60)
    }

    pub fn form_redis_key_single(&self, key: &str) -> String {
        format!("{}source:{}", self.redis_prefix_colon(), key)
    }

    pub fn redis_key_list_ttl(&self) -> Option<u64> {
        Some(60 * 60)
    }

    pub fn form_redis_key_list(&self) -> String {
        format!("{}source:list", self.redis_prefix_colon())
    }

    pub async fn delete_list_cache(&self) -> Result<(), Error> {
        if let Some(redis_pool) = &self.redis_pool {
            let _ = delete_key(
                &redis_pool,
                self.form_redis_key_list().as_str()
            ).await?;
        }
        Ok(())
    }
}


#[async_trait]
impl SourceServiceInterface for SourceService {
    async fn get(&self, source_get_command: SourceGetCommand) -> Result<Option<SourceResponse>, Error> {
        if let Some(redis_pool) = &self.redis_pool {
            let source_cache: Option<SourceResponse> = get_key(&redis_pool, self.form_redis_key_single(&source_get_command.id).as_str()).await?;
            if let Some(source_cache) = source_cache {
                return Ok(Some(source_cache));
            }
        }
        
        let source = self.source_repo.get_source(source_get_command.id.as_str()).await;
        match source {
            Ok(source) => {
                match source {
                    Some(source) => {
                        let source_response = SourceResponse::from(source);
                        if let Some(redis_pool) = &self.redis_pool {
                            let _ = set_key(
                                &redis_pool, 
                                self.form_redis_key_single(&source_get_command.id).as_str(), 
                                &source_response,
                                self.redis_key_single_ttl()
                            ).await?;
                        }
                        Ok(Some(source_response))
                    },
                    None => Ok(None)
                }
            }
            Err(_) => Err(Error::msg("Error while getting source from database"))
        }
    }

    async fn create(&self, source_create_command: SourceCreateCommand) -> Result<SourceResponse, Error> {
        let source_create = Source::new(source_create_command.name, source_create_command.website);
        let source = self.source_repo.add_source(source_create).await;
        match source {
            Ok(source) => {
                let source_response = SourceResponse::from(source);
                if let Some(redis_pool) = &self.redis_pool {
                    let _ = set_key(
                        &redis_pool, 
                        self.form_redis_key_single(&source_response.name).as_str(), 
                        &source_response,
                        self.redis_key_single_ttl()
                    ).await?;
                    self.delete_list_cache().await?;
                }
                Ok(source_response)
            },
            Err(_) => Err(Error::msg("Error while creating source in database"))
        }
    }

    async fn update(&self, source_update_command: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error> {
        let source_update = Source::new(source_update_command.name, source_update_command.website);
        let source = self.source_repo.update_source(source_update.name.clone().as_str(), source_update).await;
        match source {
            Ok(source) => {
                match source {
                    Some(source) => {
                        let source_response = SourceResponse::from(source);
                        if let Some(redis_pool) = &self.redis_pool {
                            let _ = set_key(
                                &redis_pool,
                                self.form_redis_key_single(&source_response.name).as_str(),
                                &source_response,
                                self.redis_key_single_ttl()
                            ).await?;
                            self.delete_list_cache().await?;
                        }
                        Ok(Some(source_response))
                    },
                    None => Ok(None)
                }
            },
            Err(_) => Err(Error::msg("Error while updating source in database"))
        }
    }

    async fn delete(&self, source_delete_command: SourceDeleteCommand) -> Result<(), Error> {
        let result = self.source_repo.delete_source(source_delete_command.id.as_str()).await;
        if let Some(redis_pool) = &self.redis_pool {
            let _ = delete_key(
                &redis_pool, 
                self.form_redis_key_single(&source_delete_command.id).as_str()
            ).await?;
            self.delete_list_cache().await?;
        }
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Error while deleting source from database"))
        }
    }

    async fn list(&self, _: SourceListCommand) -> Result<Vec<SourceResponse>, Error> {
        if let Some(redis_pool) = &self.redis_pool {
            let sources_cache: Option<Vec<SourceResponse>> = get_key(&redis_pool, self.form_redis_key_list().as_str()).await?;
            if let Some(sources_cache) = sources_cache {
                return Ok(sources_cache);
            }
        }
        
        let sources = self.source_repo.list_sources().await;
        match sources {
            Ok(sources) => {
                let sources: Vec<SourceResponse> = sources.into_iter().map(|s| SourceResponse::from(s)).collect();
                if let Some(redis_pool) = &self.redis_pool {
                    let _ = set_key(
                        &redis_pool,
                        self.form_redis_key_list().as_str(),
                        &sources,
                        self.redis_key_list_ttl()
                    ).await?;
                }
                Ok(sources)
            },
            Err(_) => Err(Error::msg("Error while getting sources from database"))
        }
    }
}
