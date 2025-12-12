use anyhow::Result;
use neo4rs::{ConfigBuilder, Graph};
use tracing::info;
use crate::shared::configuration::AppDatabaseNeo4jConfig;

pub async fn connect(neo4j_conf: &AppDatabaseNeo4jConfig) -> Result<Graph> {
    info!("Connecting to Neo4j...");

    let cfg = ConfigBuilder::default()
        .uri(neo4j_conf.uri.as_str())
        .user(neo4j_conf.username.as_str())
        .password(neo4j_conf.password.as_str())
        .db(neo4j_conf.database.as_str())
        .build()?;

    info!("Neo4j connected successfully!");

    Ok(Graph::connect(cfg).await?)
}

