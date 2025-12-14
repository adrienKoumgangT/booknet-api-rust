use anyhow::{Error, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;

use crate::command::language_command::{LanguageCreateCommand, LanguageDeleteCommand, LanguageGetCommand, LanguageListCommand, LanguageUpdateCommand};
use crate::dto::language_dto::{LanguageResponse};
use crate::model::language_model::{Language};
use crate::repository::language_repository::{LanguageRepository, LanguageRepositoryInterface};
use crate::shared::database::redis::{delete_key, get_key, set_key};
use crate::shared::state::AppState;


#[async_trait]
pub trait LanguageServiceInterface {
    async fn get(&self, language_get_command: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn create(&self, language_create_command: LanguageCreateCommand) -> Result<LanguageResponse, Error>;
    async fn update(&self, language_update_command: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn delete(&self, language_delete_command: LanguageDeleteCommand) -> Result<(), Error>;
    async fn list(&self, _: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error>;
}


#[derive(Clone)]
pub struct LanguageService {
    language_repo: LanguageRepository,
    redis_pool: Option<Pool<RedisConnectionManager>>,
    space_name: Option<String>,
}


impl From<&AppState> for LanguageService {
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
        
        Self::new(
            LanguageRepository::new(
                app_state.mongo_client.clone(), 
                database, 
                app_state.neo4j_client.clone()
            ), 
            Option::from(app_state.redis_pool.clone()), 
            Option::from(space_name)
        )
    }
}

impl LanguageService {
    pub fn new(language_repo: LanguageRepository, redis_pool: Option<Pool<RedisConnectionManager>>, space_name: Option<String>) -> Self {
        Self {
            language_repo,
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
        format!("{}language:{}", self.redis_prefix_colon(), key)
    }

    pub fn redis_key_list_ttl(&self) -> Option<u64> {
        Some(60 * 60)
    }

    pub fn form_redis_key_list(&self) -> String {
        format!("{}language:list", self.redis_prefix_colon())
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
impl LanguageServiceInterface for LanguageService {
    async fn get(&self, language_get_command: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error> {
        if let Some(redis_pool) = &self.redis_pool {
            let language_cache: Option<LanguageResponse> = get_key(&redis_pool, self.form_redis_key_single(language_get_command.id.as_str()).as_str()).await?;
            if let Some(language_cache) = language_cache {
                return Ok(Some(language_cache));
            }
        }
        
        let language = self.language_repo.get_language(language_get_command.id.as_str()).await;
        match language {
            Ok(language) => {
                match language {
                    Some(language) => {
                        let language_response = LanguageResponse::from(language);
                        if let Some(redis_pool) = &self.redis_pool {
                            let _ = set_key(
                                &redis_pool, 
                                self.form_redis_key_single(language_get_command.id.as_str()).as_str(), 
                                &language_response,
                                self.redis_key_single_ttl()
                            ).await?;
                        }
                        Ok(Some(language_response))
                    },
                    None => Ok(None)
                }
            },
            Err(_) => Err(Error::msg("Error while getting language from database"))
        }
    }

    async fn create(&self, language_create_command: LanguageCreateCommand) -> Result<LanguageResponse, Error> {
        let language_create = Language::new(language_create_command.code, language_create_command.name);
        let language = self.language_repo.add_language(language_create).await;
        match language {
            Ok(language) => {
                let language_response = LanguageResponse::from(language);
                if let Some(redis_pool) = &self.redis_pool {
                    let _ = set_key(
                        &redis_pool, 
                        self.form_redis_key_single(&language_response.code).as_str(), 
                        &language_response,
                        self.redis_key_single_ttl()
                    ).await?;
                    self.delete_list_cache().await?;
                }
                Ok(language_response)
            },
            Err(_) => Err(Error::msg("Error while creating language in database"))
        }
    }

    async fn update(&self, language_update_command: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error> {
        let language_update = Language::new(language_update_command.code, language_update_command.name);
        let language = self.language_repo.update_language(language_update.code.clone().as_str(), language_update).await;
        match language {
            Ok(language) => {
                match(language) {
                    Some(language) => {
                        let language_response = LanguageResponse::from(language);
                        if let Some(redis_pool) = &self.redis_pool {
                            let _ = set_key(
                                &redis_pool, 
                                self.form_redis_key_single(&language_response.code).as_str(), 
                                &language_response,
                                self.redis_key_single_ttl()
                            ).await?;
                            self.delete_list_cache().await?;
                        }
                        Ok(Some(language_response))
                    },
                    None => Ok(None)
                }
            },
            Err(_) => Err(Error::msg("Error while updating language in database"))
        }
    }

    async fn delete(&self, language_delete_command: LanguageDeleteCommand) -> Result<(), Error> {
        let result = self.language_repo.delete_language(language_delete_command.id.as_str()).await;
        if let Some(redis_pool) = &self.redis_pool {
            let _ = delete_key(
                &redis_pool, 
                self.form_redis_key_single(language_delete_command.id.as_str()).as_str()
            ).await?;
            self.delete_list_cache().await?;
        }
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::msg("Error while deleting language from database"))
        }
    }

    async fn list(&self, _: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error> {
        if let Some(redis_pool) = &self.redis_pool {
            let languages_cache: Option<Vec<LanguageResponse>> = get_key(&redis_pool, self.form_redis_key_list().as_str()).await?;
            if let Some(languages_cache) = languages_cache {
                return Ok(languages_cache);
            }
        }
        
        let languages = self.language_repo.list_languages().await;
        match languages {
            Ok(languages) => {
                let languages: Vec<LanguageResponse> = languages.into_iter().map(|l| LanguageResponse::from(l)).collect();
                if let Some(redis_pool) = &self.redis_pool {
                    let _ = set_key(
                        &redis_pool, 
                        self.form_redis_key_list().as_str(), 
                        &languages,
                        self.redis_key_list_ttl()
                    ).await?;
                }
                Ok(languages)
            },
            Err(_) => Err(Error::msg("Error while listing languages from database"))
        }
    }
}
