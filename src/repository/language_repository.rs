use anyhow::{Error, Result};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Database, Collection,
};
use neo4rs::{query, Graph};
use tracing::Instrument;
use crate::model::language_model::Language;
use crate::shared::logging::log::TimePrinter;

#[async_trait]
pub trait LanguageRepositoryInterface {
    async fn get_language(&self, language_id: &str) -> Result<Option<Language>, Error>;
    async fn add_language(&self, language: Language) -> Result<Language, Error>;
    async fn update_language(&self, language_id: &str, language: Language) -> Result<Option<Language>, Error>;
    async fn delete_language(&self, language_id: &str) -> Result<(), Error>;
    async fn list_languages(&self) -> Result<Vec<Language>, Error>;
}


#[derive(Clone)]
pub struct LanguageRepository {
    pub mongo_client: Client,
    pub language_collection: Collection<Language>,
    pub neo4j_client: Graph,
}

impl LanguageRepository {
    pub fn new(client: Client, mongo_database: Database, neo4j_client: Graph) -> Self {
        Self {
            mongo_client: client,
            language_collection: mongo_database.collection("language"),
            neo4j_client,
        }
    }
}

#[async_trait]
impl LanguageRepositoryInterface for LanguageRepository {
    async fn get_language(&self, language_id: &str) -> Result<Option<Language>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [LANGUAGE] [GET] id: {} ",
            language_id
        ));

        let language = self.language_collection.find_one(doc! {"_id": language_id}).await;
        match language {
            Ok(language) => {
                timer.log();
                Ok(language)
            }
            Err(e) => {
                timer.error_with_message(&format!("Error retrieving language with id {}: {}", language_id, e));
                Err(Error::msg(format!("Error retrieving language with id {}: {}", language_id, e)))
            }
        }
    }

    async fn add_language(&self, language: Language) -> Result<Language, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [LANGUAGE] [ADD] language: {:?} ",
            language
        ));

        let mut neo4j_tx = self.neo4j_client.start_txn().await?;
        neo4j_tx.run(
            query("CREATE (l:Language {code: $code, name: $name}) RETURN l})")
                .param("code", language.code.clone())
                .param("name", language.name.clone())
        ).await?;

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let doc_lang = Language {
            code: language.code.clone(),
            name: language.name.clone(),
        };

        if let Err(e) = self.language_collection
            .insert_one(doc_lang.clone())
            .session(&mut mongo_session)
            .await
        {
            let _ = mongo_session.abort_transaction().await;
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Error adding language: {}", e));
            return Err(Error::msg(format!("Error adding language: {}", e)));
        }

        if let Err(e) = mongo_session.commit_transaction().await {
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Error adding language: {}", e));
            return Err(Error::msg(format!("Error adding language: {}", e)));
        }

        if let Err(e) = neo4j_tx.commit().await {
            let _ = self.language_collection.delete_one(doc! { "_id": language.code }).await;

            timer.error_with_message(&format!("Error adding language: {}", e));
            return Err(Error::msg(format!("Error adding language: {}", e)));
        }

        timer.log();
        Ok(language)
    }

    async fn update_language(&self, language_id: &str, language: Language) -> Result<Option<Language>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [LANGUAGE] [UPDATE] language: {:?} ",
            language
        ));

        let mut neo4j_tx = self.neo4j_client.start_txn().await?;
        neo4j_tx.run(
            query("MATCH (l:Language {code: $code}) SET l.name = $name}) RETURN l")
                .param("code", language_id)
                .param("name", language.name.clone())
        ).await?;

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let res = self.language_collection
            .update_one(
                doc! { "_id": language_id },
                doc! { "$set": { "name": language.name } }
            )
            .session(&mut mongo_session)
            .await?;

        if res.modified_count == 0 {
            let _ = mongo_session.abort_transaction().await;
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Language with id {} not found", language_id));
            return Err(Error::msg(format!("Language with id {} not found", language_id)));
        }

        if let Err(e) = mongo_session.commit_transaction().await {
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Error updating language: {}", e));
            return Err(Error::msg(format!("Error updating language: {}", e)));
        }

        timer.log();
        Ok(self.get_language(language_id).await?)
    }

    async fn delete_language(&self, language_id: &str) -> Result<(), Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [LANGUAGE] [DELETE] id: {} ",
            language_id
        ));

        let mut neo4j_tx = self.neo4j_client.start_txn().await?;
        neo4j_tx.run(query("MATCH (l:Language {code: $code}) DETACH DELETE l")
            .param("code", language_id)
        ).await?;

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let res = self.language_collection
            .delete_one(doc! {"_id": language_id})
            .session(&mut mongo_session)
            .await?;

        if res.deleted_count == 0 {
            let _ = mongo_session.abort_transaction().await;
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Error deleting language with id: {}", language_id));
            return Err(Error::msg(format!("Error deleting language with id: {}", language_id )));
        }

        if let Err(e) = mongo_session.commit_transaction().await {
            let _ = neo4j_tx.rollback().await;

            timer.error_with_message(&format!("Error deleting language: {}", e));
            return Err(Error::msg(format!("Error deleting language: {}", e)));
        }

        timer.log();
        Ok(())
    }

    async fn list_languages(&self) -> Result<Vec<Language>, Error> {
        let timer = TimePrinter::with_message("[REPOSITORY] [LANGUAGE] [LIST]");

        let result = self.language_collection.find(doc! {}).await;

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
                timer.error_with_message(&format!("Error fetching languages: {}", e));
                Err(Error::msg(format!("Error fetching languages: {}", e)))
            }
        }
    }
}

