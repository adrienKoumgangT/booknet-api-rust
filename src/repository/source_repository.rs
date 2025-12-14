use anyhow::{Error, Result};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Database, Collection,
};
use crate::model::source_model::Source;
use crate::shared::logging::log::TimePrinter;

#[async_trait]
pub trait SourceRepositoryInterface {
    async fn get_source(&self, source_id: &str) -> Result<Option<Source>, Error>;

    async fn add_source(&self, source: Source) -> Result<Source, Error>;

    async fn update_source(&self, source_id: &str, source: Source) -> Result<Option<Source>, Error>;

    async fn delete_source(&self, source_id: &str) -> Result<(), Error>;

    async fn list_sources(&self) -> Result<Vec<Source>, Error>;
}


#[derive(Clone)]
pub struct SourceRepository {
    pub source_collection: Collection<Source>,
}


impl SourceRepository {
    pub fn new(mongo_database: Database) -> Self {
        Self {
            source_collection: mongo_database.collection("source"),
        }
    }
}

#[async_trait]
impl SourceRepositoryInterface for SourceRepository {
    async fn get_source(&self, source_id: &str) -> Result<Option<Source>, Error> {
        // let source = self.source_collection.find_one(doc! {"_id": ObjectId::parse_str(source_id)?}).await?;

        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [SOURCE] [GET] id: {} ",
            source_id
        ));

        let source = self.source_collection.find_one(doc! {"_id": source_id}).await;
        match source {
            Ok(source) => {
                timer.log();
                Ok(source)
            }
            Err(e) => {
                timer.error_with_message(&format!("Error retrieving source with id {}: {}", source_id, e));
                Err(Error::msg(format!("Error retrieving source with id {}: {}", source_id, e)))
            }
        }
    }

    async fn add_source(&self, source: Source) -> Result<Source, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [SOURCE] [ADD] source: {:?} ",
            source
        ));

        let result = self.source_collection.insert_one(source).await;

        match result {
            Ok(insert_one_result) => {
                let source = self.source_collection.find_one(doc! {"_id": insert_one_result.inserted_id}).await;

                match source {
                    Ok(source) => {
                        timer.log();
                        Ok(source.unwrap())
                    }
                    Err(e) => {
                        timer.error_with_message(&format!("Error retrieving added source: {}", e));
                        Err(Error::msg(format!("Error retrieving added source: {}", e)))
                    }
                }
            }
            Err(e) => {
                timer.error_with_message(&format!("Error adding source: {}", e));
                Err(Error::msg(format!("Error adding source: {}", e)))
            }
        }
    }

    async fn update_source(&self, source_id: &str, source: Source) -> Result<Option<Source>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [SOURCE] [UPDATE] source: {:?} ",
            source
        ));

        let filter = doc! {"_id": source_id};
        let update = doc! {"$set": doc! {"website": &source.website}};
        let result = self.source_collection.update_one(filter, update).await;

        match result {
            Ok(update_result) => {
                if update_result.matched_count > 0 {
                    let source = self.source_collection.find_one(doc! {"_id": source_id}).await;

                    match source {
                        Ok(source) => {
                            timer.log();
                            Ok(source)
                        }
                        Err(e) => {
                            timer.error_with_message(&format!("Error retrieving updated source: {}", e));
                            Err(Error::msg(format!("Error retrieving updated source: {}", e)))
                        }
                    }
                } else {
                    timer.log();
                    Ok(None)
                }
            },
            Err(e) => {
                timer.error_with_message(&format!("Error updating source: {}", e));
                Err(Error::msg(format!("Error updating source: {}", e)))
            }
        }
    }

    async fn delete_source(&self, source_id: &str) -> Result<(), Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [SOURCE] [DELETE] id: {} ",
            source_id
        ));

        let result = self.source_collection.delete_one(doc! {"_id": source_id}).await;

        match result {
            Ok(delete_result) => {
                if delete_result.deleted_count > 0 {
                    timer.log();
                    Ok(())
                } else {
                    timer.error_with_message(&format!("Source with id {} not found", source_id));
                    Err(Error::msg(format!("Source with id {} not found", source_id)))
                }
            },
            Err(e) => {
                timer.error_with_message(&format!("Error deleting source: {}", e));
                Err(Error::msg(format!("Error deleting source: {}", e)))
            }
        }
    }

    async fn list_sources(&self) -> Result<Vec<Source>, Error> {
        let timer = TimePrinter::with_message("[REPOSITORY] [SOURCE] [LIST]");

        let result = self.source_collection.find(doc! {}).await;

        match result {
            Ok(mut cursor) => {
                let mut out = Vec::new();
                while let Some(item) = cursor.next().await {
                    out.push(item?);
                }
                timer.log();
                Ok(out)
            },
            Err(e) => {
                timer.error_with_message(&format!("Error fetching sources: {}", e));
                Err(Error::msg(format!("Error fetching sources: {}", e)))
            }
        }
    }
}
