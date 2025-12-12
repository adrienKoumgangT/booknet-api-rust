use anyhow::Result;
use mongodb::{
    bson::doc,
    options::ClientOptions,
    Client
};
use tracing::info;
use crate::shared::configuration::AppDatabaseMongoDBConfig;

pub async fn connect(mongodb_config: &AppDatabaseMongoDBConfig) -> Result<Client> {
    info!("Connecting to Mongodb...");

    let client_options = ClientOptions::parse(&mongodb_config.uri).await?;

    let client = Client::with_options(client_options)?;

    // client.database(mongodb_config.database.as_str())
    client.database("admin")
        .run_command(doc! {"ping": 1})
        .await?;

    info!("Mongodb connected successfully!");

    Ok(client)
}


