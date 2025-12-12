use anyhow::Result;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use mongodb::Client;
use neo4rs::Graph;
use tracing::info;
use crate::shared::configuration::AppConfig;
use crate::shared::database::mongodb as my_mongodb;
use crate::shared::database::neo4j as my_neo4j;
use crate::shared::database::redis as my_redis;
// use crate::shared::metrics::prometheus::Metrics;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub mongo_client: Client,
    pub neo4j_client: Graph,
    pub redis_pool: Pool<RedisConnectionManager>,

    // pub metrics: Metrics,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let config_clone = config.clone();

        info!("Initializing application state...");
        
        let redis_pool = my_redis::connect(&config_clone.database.redis.unwrap()).await?;
        let mongo_client = my_mongodb::connect(&config_clone.database.mongo.unwrap()).await?;
        let neo4j_client = my_neo4j::connect(&config_clone.database.neo4j.unwrap()).await?;
        // let metrics = Metrics::new();

        info!("Application state initialized successfully!");

        Ok(Self {
            config,
            mongo_client,
            neo4j_client,
            redis_pool,
            // metrics,
        })
    }
}
