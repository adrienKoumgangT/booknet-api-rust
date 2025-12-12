use anyhow::Result;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use tracing::info;
use crate::shared::configuration::AppDatabaseRedisConfig;
use crate::shared::logging::log::TimePrinter;

pub type RedisDatabase = Pool<RedisConnectionManager>;

pub async fn connect(redis_config: &AppDatabaseRedisConfig) -> Result<RedisDatabase> {
    info!("Connecting to Redis...");

    let manager = RedisConnectionManager::new(redis_config.uri.as_str())?;
    let pool = Pool::builder().build(manager).await?;

    // Test connection
    let mut conn = pool.get().await?;
    let _: () = conn.set("health_check", "ok").await?;

    info!("Redis connected successfully!");
    Ok(pool.clone())
}

pub async fn set_key<T: serde::Serialize>(
    pool: &RedisDatabase,
    key: &str,
    value: &T,
    ttl_seconds: Option<u64>,
) -> Result<()> {
    let timer = TimePrinter::with_message(&format!(
        "[REDIS] [SET] Key: {} ",
        key.to_string()
    ));

    let mut conn = pool.get().await?;
    let serialized = serde_json::to_string(value)?;

    if let Some(ttl) = ttl_seconds {
        let _: () = conn.set_ex(key, serialized, ttl).await?;
    } else {
        let _: () = conn.set(key, serialized).await?;
    }

    timer.log();

    Ok(())
}

pub async fn get_key<T: serde::de::DeserializeOwned>(
    pool: &RedisDatabase,
    key: &str,
) -> Result<Option<T>> {
    let timer = TimePrinter::with_message(&format!(
        "[REDIS] [GET] Key: {} ",
        key.to_string()
    ));

    let mut conn = pool.get().await?;
    let result: Option<String> = conn.get(key).await?;

    match result {
        Some(data) => {
            let deserialized = serde_json::from_str(&data)?;
            timer.log();
            Ok(Some(deserialized))
        }
        None => {
            timer.warning();
            Ok(None)
        },
    }
}

pub async fn delete_key(pool: &RedisDatabase, key: &str) -> Result<()> {
    let timer = TimePrinter::with_message(&format!(
        "[REDIS] [DELETE] Key: {} ",
        key.to_string()
    ));

    let mut conn = pool.get().await?;
    let _: () = conn.del(key).await?;

    timer.log();
    Ok(())
}