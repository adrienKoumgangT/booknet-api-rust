use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigJWT {
    pub private_secret_pem_path: Option<String>,
    pub public_secret_pem_path: String, // HS256 secret
    pub issuer: String, // "booknet"
    pub audience: String, // "all-services" or service name
    pub expires_in_minutes: i64,
    pub kid: Option<String>, // key id shown in the JWT header
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabaseMongoDBConfig {
    pub uri: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabaseRedisConfig {
    pub uri: String,
    pub default_ttl: Option<u64>, // in seconds
    pub max_connections: Option<u32>,
    pub app_space_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabaseNeo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
    pub encrypted: bool,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabaseConfig {
    pub mongo: Option<AppDatabaseMongoDBConfig>,
    pub redis: Option<AppDatabaseRedisConfig>,
    pub neo4j: Option<AppDatabaseNeo4jConfig>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub is_prod: bool,

    pub log_level: String,

    pub jwt: AppConfigJWT,

    pub database: AppDatabaseConfig,

    pub bind_addr: String,
    pub metrics_addr: String,
}


impl AppConfig {

    /// Loads the configuration from environment variables.
    /// It will first attempt to load a `.env` file if present.
    pub fn default() -> Result<Self> {
        // Load the .env file if it exists
        dotenvy::dotenv().ok();

        let bind_addr = get_env("BIND_ADDR")?;
        let metrics_addr = get_env("METRICS_ADDR")?;

        // Use a helper function to get required env vars
        let is_prod = matches!(get_env("APP_ENV")?.to_ascii_lowercase().as_str(), "prod" | "production");

        let log_level = get_env("LOG_LEVEL").ok().unwrap_or_else(|| "info".to_string());

        let jwt_private_secret_pem_path = get_env("JWT_RSA_PRIVATE_KEY_PATH").ok();
        let jwt_public_secret_pem_path = get_env("JWT_RSA_PUBLIC_KEY_PATH")?;
        let jwt_issuer = get_env("JWT_ISSUER")?;
        let jwt_audience = get_env("JWT_AUDIENCE")?;
        let jwt_expires_in_minutes = get_env("JWT_EXPIRES_IN_MINUTES")?.trim().parse::<i64>()?;
        let jwt_kid = get_env("JWT_KID").ok();

        let jwt = AppConfigJWT {
            private_secret_pem_path: jwt_private_secret_pem_path,
            public_secret_pem_path: jwt_public_secret_pem_path,
            issuer: jwt_issuer,
            audience: jwt_audience,
            expires_in_minutes: jwt_expires_in_minutes,
            kid: jwt_kid,
        };

        let mongo_url = get_env("MONGO_URL").ok();
        let mongo = match mongo_url {
            Some(url) => {
                let mongo_database = get_env("MONGO_DATABASE")?;
                Some(AppDatabaseMongoDBConfig {
                    uri: url,
                    database: mongo_database,
                })
            },
            None => None,
        };

        let redis_url = get_env("REDIS_URL").ok();
        let redis = match redis_url {
            Some(url) => {
                let redis_default_ttl = get_env("REDIS_DEFAULT_TTL").ok()
                    .map(|ttl| ttl.trim().parse::<u64>().unwrap());
                let redis_max_connections = get_env("REDIS_MAX_CONNECTIONS").ok()
                    .map(|ttl| ttl.trim().parse::<u32>().unwrap());
                let redis_app_space_name = get_env("REDIS_APP_SPACE_NAME").ok();
                Some(AppDatabaseRedisConfig {
                    uri: url,
                    default_ttl: redis_default_ttl,
                    max_connections: redis_max_connections,
                    app_space_name: redis_app_space_name
                })
            },
            None => None,
        };

        let neo4j_url = get_env("NEO4J_URL").ok();
        let neo4j = match neo4j_url {
            Some(url) => {
                let neo4j_username = get_env("NEO4J_USERNAME")?;
                let neo4j_password = get_env("NEO4J_PASSWORD")?;
                let neo4j_encrypted = get_env("NEO4J_ENCRYPTED")?.trim().parse::<bool>()?;
                let neo4j_database = get_env("NEO4J_DATABASE")?;
                Some(AppDatabaseNeo4jConfig {
                    uri: url,
                    username: neo4j_username,
                    password: neo4j_password,
                    encrypted: neo4j_encrypted,
                    database: neo4j_database,
                })
            },
            None => None,
        };

        let database = AppDatabaseConfig {
            mongo,
            redis,
            neo4j,
        };

        Ok(AppConfig {
            is_prod,

            log_level,

            jwt,

            database,

            bind_addr,
            metrics_addr,
        })
    }
}


/// Helper to read an environment variable and return an error if it's not set.
fn get_env(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("Missing required environment variable: {}", key))
}

