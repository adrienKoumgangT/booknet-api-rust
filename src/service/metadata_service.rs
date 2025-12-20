use anyhow::{Error, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;

use crate::command::{
    genre_command::{
        GenreCreateCommand, GenreDeleteCommand, GenreGetCommand, GenreListCommand, GenreUpdateCommand
    },
    language_command::{
        LanguageCreateCommand, LanguageDeleteCommand, LanguageGetCommand, LanguageListCommand, LanguageUpdateCommand
    },
    publisher_command::{
        PublisherCreateCommand, PublisherDeleteCommand, PublisherGetCommand, PublisherListCommand, PublisherUpdateCommand
    },
    source_command::{
        SourceCreateCommand, SourceDeleteCommand, SourceGetCommand, SourceListCommand, SourceUpdateCommand
    }
};
use crate::dto::{
    genre_dto::GenreResponse,
    language_dto::LanguageResponse,
    publisher_dto::PublisherResponse,
    source_dto::SourceResponse
};
use crate::model::metadata_model::{Metadata, MetadataKey};
use crate::repository::metadata_repository::{MetadataRepository, MetadataRepositoryInterface};
use crate::shared::database::redis::{delete_key, get_key, set_key};
use crate::shared::state::AppState;


#[async_trait]
pub trait MetadataServiceInterface {

    // Genre
    async fn get_genre(&self, cmd: GenreGetCommand) -> Result<Option<GenreResponse>, Error>;
    async fn create_genre(&self, cmd: GenreCreateCommand) -> Result<GenreResponse, Error>;
    async fn update_genre(&self, cmd: GenreUpdateCommand) -> Result<Option<GenreResponse>, Error>;
    async fn delete_genre(&self, cmd: GenreDeleteCommand) -> Result<(), Error>;
    async fn list_genres(&self, _: GenreListCommand) -> Result<Vec<GenreResponse>, Error>;

    // Language
    async fn get_language(&self, cmd: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn create_language(&self, cmd: LanguageCreateCommand) -> Result<LanguageResponse, Error>;
    async fn update_language(&self, cmd: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error>;
    async fn delete_language(&self, cmd: LanguageDeleteCommand) -> Result<(), Error>;
    async fn list_languages(&self, _: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error>;
    
    // Publisher
    async fn get_publisher(&self, cmd: PublisherGetCommand) -> Result<Option<PublisherResponse>, Error>;
    async fn create_publisher(&self, cmd: PublisherCreateCommand) -> Result<PublisherResponse, Error>;
    async fn update_publisher(&self, cmd: PublisherUpdateCommand) -> Result<Option<PublisherResponse>, Error>;
    async fn delete_publisher(&self, cmd: PublisherDeleteCommand) -> Result<(), Error>;
    async fn list_publishers(&self, _: PublisherListCommand) -> Result<Vec<PublisherResponse>, Error>;

    // Source
    async fn get_source(&self, cmd: SourceGetCommand) -> Result<Option<SourceResponse>, Error>;
    async fn create_source(&self, cmd: SourceCreateCommand) -> Result<SourceResponse, Error>;
    async fn update_source(&self, cmd: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error>;
    async fn delete_source(&self, cmd: SourceDeleteCommand) -> Result<(), Error>;
    async fn list_sources(&self, _: SourceListCommand) -> Result<Vec<SourceResponse>, Error>;
}


#[derive(Clone)]
pub struct MetadataService {
    metadata_repo: MetadataRepository,
    redis_pool: Option<Pool<RedisConnectionManager>>,
    space_name: Option<String>,
}


impl From<&AppState> for MetadataService {
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
            MetadataRepository::new(
                app_state.mongo_client.clone(),
                database,
                app_state.neo4j_client.clone()
            ),
            Some(app_state.redis_pool.clone()),
            Some(space_name),
        )
    }
}


impl MetadataService {
    pub fn new(
        metadata_repo: MetadataRepository,
        redis_pool: Option<Pool<RedisConnectionManager>>,
        space_name: Option<String>
    ) -> Self {
        MetadataService { metadata_repo, redis_pool, space_name }
    }

    // --- Redis Helper Methods ---

    fn redis_prefix_colon(&self) -> String {
        self.space_name
            .as_deref()
            .map(|s| format!("{s}:"))
            .unwrap_or_default()
    }

    fn redis_ttl(&self) -> u64 { 60 * 60 } // 1 hour

    // Generates: "booknet:source:google_books" or "booknet:language:en"
    fn cache_key(&self, kind: &str, key: &str) -> String {
        format!("{}{}:{}", self.redis_prefix_colon(), kind, key)
    }

    // Generates: "booknet:source:list" or "booknet:genre:list"
    fn list_cache_key(&self, kind: &str) -> String {
        format!("{}{}:list", self.redis_prefix_colon(), kind)
    }

    async fn clear_list_cache(&self, kind: &str) -> Result<(), Error> {
        if let Some(pool) = &self.redis_pool {
            let _ = delete_key(pool, &self.list_cache_key(kind)).await?;
        }
        Ok(())
    }


    // --- Generic Internal Logic (avoids code duplication) ---


    async fn _get(&self, key: MetadataKey) -> Result<Option<Metadata>, Error> {
        let cache_key = self.cache_key(key.kind(), key.key());

        if let Some(pool) = &self.redis_pool {
            let cached: Option<Metadata> = get_key(pool, &cache_key).await?;
            if let Some(meta) = cached {
                return Ok(Some(meta));
            }
        }

        let result = self.metadata_repo.find_by_key(key).await?;

        if let Some(meta) = &result {
            if let Some(pool) = &self.redis_pool {
                let _ = set_key(pool, &cache_key, meta, Some(self.redis_ttl())).await?;
            }
        }

        Ok(result)
    }


    async fn _create(&self, meta: Metadata) -> Result<Metadata, Error> {
        let kind = meta.kind();
        let key_str = meta.key().to_string(); // clone strictly for string generation

        let created = self.metadata_repo.insert(meta).await?;

        if let Some(pool) = &self.redis_pool {
            let _ = set_key(
                pool,
                &self.cache_key(kind, &key_str),
                &created,
                Some(self.redis_ttl())
            ).await?;

            let _ = delete_key(pool, &self.list_cache_key(kind)).await?;
        }

        Ok(created)
    }


    async fn _update(&self, meta: Metadata) -> Result<Option<Metadata>, Error> {
        let kind = meta.kind();
        let key_str = meta.key().to_string();

        let updated = self.metadata_repo.update(meta).await?;

        if let Some(result) = &updated {
            if let Some(pool) = &self.redis_pool {
                let _ = set_key(
                    pool,
                    &self.cache_key(kind, &key_str),
                    result,
                    Some(self.redis_ttl())
                ).await?;

                let _ = delete_key(pool, &self.list_cache_key(kind)).await?;
            }
        }

        Ok(updated)
    }


    async fn _delete(&self, key: MetadataKey) -> Result<(), Error> {
        let kind = key.kind();
        let key_str = key.key().to_string();

        self.metadata_repo.delete(key).await?;

        if let Some(pool) = &self.redis_pool {
            let _ = delete_key(pool, &self.cache_key(kind, &key_str)).await?;
            let _ = delete_key(pool, &self.list_cache_key(kind)).await?;
        }

        Ok(())
    }


    async fn _list(&self, kind: &str) -> Result<Vec<Metadata>, Error> {
        let cache_key = self.list_cache_key(kind);

        if let Some(pool) = &self.redis_pool {
            let cached: Option<Vec<Metadata>> = get_key(pool, &cache_key).await?;
            if let Some(list) = cached {
                return Ok(list);
            }
        }

        let list = self.metadata_repo.find_all_by_type(kind).await?;

        if let Some(pool) = &self.redis_pool {
            let _ = set_key(pool, &cache_key, &list, Some(self.redis_ttl())).await?;
        }

        Ok(list)
    }
}

#[async_trait]
impl MetadataServiceInterface for MetadataService {
    

    

    // --- Genre Implementation ---

    async fn get_genre(&self, cmd: GenreGetCommand) -> Result<Option<GenreResponse>, Error> {
        let metadata = self._get(MetadataKey::Genre { name: cmd.id }).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(GenreResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while getting metadata from database"))
        }
    }

    async fn create_genre(&self, cmd: GenreCreateCommand) -> Result<GenreResponse, Error> {
        let meta = Metadata::new_genre(cmd.name, cmd.description);
        let metadata = self._create(meta).await;
        match metadata {
            Ok(meta) => Ok(GenreResponse::from(meta)),
            Err(_) => Err(Error::msg("Error while creating metadata in database"))
        }
    }

    async fn update_genre(&self, cmd: GenreUpdateCommand) -> Result<Option<GenreResponse>, Error> {
        let meta = Metadata::new_genre(cmd.name, cmd.description);
        let metadata = self._update(meta).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(GenreResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while updating metadata in database"))
        }
    }

    async fn delete_genre(&self, cmd: GenreDeleteCommand) -> Result<(), Error> {
        self._delete(MetadataKey::Genre { name: cmd.id }).await
    }

    async fn list_genres(&self, _: GenreListCommand) -> Result<Vec<GenreResponse>, Error> {
        let genres = self._list("genre").await;
        match genres {
            Ok(genres) => Ok(genres.into_iter().map(GenreResponse::from).collect()),
            Err(_) => Err(Error::msg("Error while listing genres from database"))
        }
    }


    // --- Language Implementation ---

    async fn get_language(&self, cmd: LanguageGetCommand) -> Result<Option<LanguageResponse>, Error> {
        let metadata = self._get(MetadataKey::Language { code: cmd.id.to_string() }).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(LanguageResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while getting metadata from database"))
        }
    }

    async fn create_language(&self, cmd: LanguageCreateCommand) -> Result<LanguageResponse, Error> {
        let meta = Metadata::new_language(cmd.code, cmd.name);
        let metadata = self._create(meta).await;
        match metadata {
            Ok(meta) => Ok(LanguageResponse::from(meta)),
            Err(_) => Err(Error::msg("Error while creating metadata in database"))
        }
    }

    async fn update_language(&self, cmd: LanguageUpdateCommand) -> Result<Option<LanguageResponse>, Error> {
        let meta = Metadata::new_language(cmd.code, cmd.name);
        let metadata = self._update(meta).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(LanguageResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while updating metadata in database"))
        }
    }

    async fn delete_language(&self, cmd: LanguageDeleteCommand) -> Result<(), Error> {
        self._delete(MetadataKey::Language { code: cmd.id }).await
    }

    async fn list_languages(&self, _: LanguageListCommand) -> Result<Vec<LanguageResponse>, Error> {
        let languages = self._list("language").await;
        match languages {
            Ok(languages) => Ok(languages.into_iter().map(LanguageResponse::from).collect()),
            Err(_) => Err(Error::msg("Error while listing languages from database"))
        }
    }
    
    
    // --- Publisher Implementation ---
    
    async fn get_publisher(&self, cmd: PublisherGetCommand) -> Result<Option<PublisherResponse>, Error> {
        let metadata = self._get(MetadataKey::Publisher { name: cmd.id }).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(PublisherResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while getting metadata from database"))
        }
    }

    async fn create_publisher(&self, cmd: PublisherCreateCommand) -> Result<PublisherResponse, Error> {
        let meta = Metadata::new_publisher(cmd.name, cmd.website);
        let metadata = self._create(meta).await;
        match metadata {
            Ok(meta) => Ok(PublisherResponse::from(meta)),
            Err(_) => Err(Error::msg("Error while creating metadata in database"))
        }
    }

    async fn update_publisher(&self, cmd: PublisherUpdateCommand) -> Result<Option<PublisherResponse>, Error> {
        let meta = Metadata::new_publisher(cmd.name, cmd.website);
        let metadata = self._update(meta).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(PublisherResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while updating metadata in database"))
        }
    }
    
    async fn delete_publisher(&self, cmd: PublisherDeleteCommand) -> Result<(), Error> {
        self._delete(MetadataKey::Publisher { name: cmd.id }).await
    }
    
    async fn list_publishers(&self, _: PublisherListCommand) -> Result<Vec<PublisherResponse>, Error> {
        let publishers = self._list("publisher").await;
        match publishers {
            Ok(publishers) => Ok(publishers.into_iter().map(PublisherResponse::from).collect()),
            Err(_) => Err(Error::msg("Error while listing publishers from database"))
        }
    }


    // --- Source Implementation ---

    async fn get_source(&self, cmd: SourceGetCommand) -> Result<Option<SourceResponse>, Error> {
        let metadata = self._get(MetadataKey::Source { name: cmd.id }).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(SourceResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while getting metadata from database"))
        }
    }

    async fn create_source(&self, cmd: SourceCreateCommand) -> Result<SourceResponse, Error> {
        let meta = Metadata::new_source(cmd.name, cmd.website);
        let metadata = self._create(meta).await;
        match metadata {
            Ok(meta) => Ok(SourceResponse::from(meta)),
            Err(_) => Err(Error::msg("Error while creating metadata in database"))
        }
    }

    async fn update_source(&self, cmd: SourceUpdateCommand) -> Result<Option<SourceResponse>, Error> {
        let meta = Metadata::new_source(cmd.name, cmd.website);
        let metadata = self._update(meta).await;
        match metadata {
            Ok(Some(meta)) => Ok(Some(SourceResponse::from(meta))),
            Ok(None) => Ok(None),
            Err(_) => Err(Error::msg("Error while updating metadata in database"))
        }
    }

    async fn delete_source(&self, cmd: SourceDeleteCommand) -> Result<(), Error> {
        self._delete(MetadataKey::Source { name: cmd.id }).await
    }

    async fn list_sources(&self, _: SourceListCommand) -> Result<Vec<SourceResponse>, Error> {
        let sources = self._list("source").await;
        match sources {
            Ok(sources) => Ok(sources.into_iter().map(SourceResponse::from).collect()),
            Err(_) => Err(Error::msg("Error while listing sources from database"))
        }
    }
}

