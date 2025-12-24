use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Database, Collection,
};
use neo4rs::{query, Graph, Query, Txn};

use crate::model::metadata_model::{Metadata, MetadataDoc, MetadataKey};
use crate::shared::logging::log::TimePrinter;
use crate::shared::repository::repository_utils::neo4j_count;

impl Metadata {
    pub fn neo4j_create_query(&self) -> Query {
        match self {
            Metadata::Genre { name, description } => query(
                "CREATE (g:Genre {name:$k, description:$description})"
            ).param("k", name.as_str()).param("description", description.as_str()),

            _ => unreachable!(),
        }
    }

    pub fn neo4j_update_query_with_count(&self) -> Query {
        match self {
            Metadata::Genre { name, description } => query(
                "MATCH (g:Genre {name:$k})
                 SET g.description = $description
                 RETURN count(g) AS n"
            ).param("k", name.as_str()).param("description", description.as_str()),

            _ => unreachable!(),
        }
    }

    pub fn neo4j_delete_query(&self) -> Query {
        match self {
            Metadata::Genre { name, .. } => query("MATCH (g:Genre {name:$id}) DETACH DELETE g")
                .param("id", name.as_str()),
            _ => unreachable!(),
        }
    }
}


impl MetadataKey {
    pub fn neo4j_delete_query_with_count(&self) -> Query {
        match self {

            MetadataKey::Genre { name } => query(
                "MATCH (g:Genre {name:$k})
                 WITH g, count(g) AS n
                 DETACH DELETE g
                 RETURN n"
            ).param("k", name.as_str()),

            _ => unreachable!(),
        }
    }
}


#[async_trait]
pub trait MetadataRepositoryInterface {
    async fn insert(&self, metadata: Metadata) -> Result<Metadata, Error>;
    async fn update(&self, metadata: Metadata) -> Result<Option<Metadata>, Error>;
    async fn delete(&self, key: MetadataKey) -> Result<(), Error>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Metadata>, Error>;
    async fn find_by_key(&self, key: MetadataKey) -> Result<Option<Metadata>, Error>;
    async fn find_all(&self) -> Result<Vec<Metadata>, Error>;
    async fn find_all_by_type(&self, metadata_type: &str) -> Result<Vec<Metadata>, Error>;
}


#[derive(Clone)]
pub struct MetadataRepository {
    pub mongo_client: Client,
    pub metadata_collection: Collection<MetadataDoc>,
    pub neo4j_client: Graph,
}

impl MetadataRepository {
    pub fn new(mongo_client: Client, mongo_database: Database, neo4j_client: Graph) -> Self {
        let metadata_collection = mongo_database.collection::<MetadataDoc>("metadata");
        MetadataRepository {
            mongo_client,
            metadata_collection,
            neo4j_client,
        }
    }
}


#[async_trait]
impl MetadataRepositoryInterface for MetadataRepository {
    async fn insert(&self, metadata: Metadata) -> Result<Metadata, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [INSERT] {:?}: {:?} ",
            metadata.kind(), metadata
        ));

        let new_doc = metadata.to_doc();
        let id = new_doc.id.clone();

        if(metadata.save_in_noe4j()) {
            let mut neo4j_tx = self.neo4j_client.start_txn().await?;
            neo4j_tx.run(metadata.neo4j_create_query()).await?;

            let mut mongo_session = self.mongo_client.start_session().await?;
            mongo_session.start_transaction().await?;

            if let Err(e) = self.metadata_collection
                .insert_one(new_doc.clone())
                .session(&mut mongo_session)
                .await
            {
                let _ = mongo_session.abort_transaction().await;
                let _ = neo4j_tx.rollback().await;
                timer.error_with_message(&format!("Error adding metadata: {}", e));
                return Err(e.into());
            }

            mongo_session.commit_transaction().await?;

            if let Err(e) = neo4j_tx.commit().await {
                let _ = self.metadata_collection.delete_one(doc! { "_id": &id }).await;
                timer.error_with_message(&format!("Error adding metadata: {}", e));
                return Err(e.into());
            }
        } else {
            let _ = self.metadata_collection.insert_one(new_doc.clone()).await?;
        }

        timer.log();
        Ok(new_doc.meta)
    }

    async fn update(&self, metadata: Metadata) -> Result<Option<Metadata>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [UPDATE] {:?}: {:?} ",
            metadata.kind(), metadata
        ));

        let id = metadata.mongo_id();
        let filter = doc! {"_id": &id };
        let update = match &metadata {
            Metadata::Source { website, .. } => doc! { "$set": { "website": website } },
            Metadata::Language { name, .. } => doc! { "$set": { "name": name } },
            Metadata::Genre { description, .. } => doc! { "$set": { "description": description } },
            Metadata::Publisher { website, .. } => doc! { "$set": { "website": website } },
        };

        if(!metadata.save_in_noe4j()) {
            let mut neo_tx = self.neo4j_client.start_txn().await?;
            let n = neo4j_count(&mut neo_tx, metadata.neo4j_update_query_with_count()).await?;
            if n == 0 {
                let _ = neo_tx.rollback().await;
                timer.error_with_message(&format!("Neo4j node not found for {}", id));
                return Err(anyhow!("Neo4j node not found for {}", id));
            }

            let mut session = self.mongo_client.start_session().await?;
            session.start_transaction().await?;

            let old = self
                .metadata_collection
                .find_one(filter.clone())
                .session(&mut session)
                .await?;

            let old = old.ok_or_else(|| anyhow!("Mongo doc not found for {}", id))?;

            self.metadata_collection
                .update_one(filter, update)
                .session(&mut session)
                .await?;

            session.commit_transaction().await?;

            if let Err(e) = neo_tx.commit().await {
                let _ = self.metadata_collection.replace_one(doc! { "_id": &id }, old).await;
                timer.error_with_message(&format!("Error adding metadata: {}", e));
                return Err(e.into());
            }
        } else {
            let update_result = self.metadata_collection
                .update_one(filter, update)
                .await?;

            if update_result.matched_count == 0 {
                timer.error_with_message(&format!("Mongo doc not found for {}", id));
                return Err(anyhow!("Mongo doc not found for {}", id));
            }
        }

        timer.log();
        Ok(Some(metadata))
    }

    async fn delete(&self, key: MetadataKey) -> Result<(), Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [DELETE] {:?}: {:?} ",
            key.kind(), key
        ));

        let id = key.mongo_id();
        let filter = doc! {"_id": &id };

        if key.save_in_noe4j() {
            let mut neo_tx = self.neo4j_client.start_txn().await?;
            let n = neo4j_count(&mut neo_tx, key.neo4j_delete_query_with_count()).await?;
            if n == 0 {
                let _ = neo_tx.rollback().await;
                timer.error_with_message(&format!("Neo4j node not found for {}", id));
                return Err(anyhow!("Neo4j node not found for {}", id));
            }

            let mut session = self.mongo_client.start_session().await?;
            session.start_transaction().await?;

            let old = self
                .metadata_collection
                .find_one(filter.clone())
                .session(&mut session)
                .await?;

            let old = old.ok_or_else(|| anyhow!("Mongo doc not found for {}", id))?;

            self.metadata_collection
                .delete_one(filter)
                .session(&mut session)
                .await?;

            session.commit_transaction().await?;

            if let Err(e) = neo_tx.commit().await {
                let _ = self.metadata_collection.insert_one(old).await;
                timer.error_with_message(&format!("Error updating metadata: {}", e));
                return Err(e.into());
            }
        } else {
            let delete_result = self.metadata_collection
                .delete_one(filter)
                .await?;

            if delete_result.deleted_count == 0 {
                timer.error_with_message(&format!("Mongo doc not found for {}", id));
                return Err(anyhow!("Mongo doc not found for {}", id));
            }
        }

        timer.log();
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Metadata>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [FIND BY ID] id: {:?} ",
            id
        ));

        let doc_opt = self.metadata_collection
            .find_one(doc! { "_id": id })
            .await?;

        // Ok(doc_opt.map(|d| d.meta))
        match doc_opt {
            Some(d) => {
                timer.log();
                Ok(Some(d.meta))
            },
            None => {
                timer.error_with_message(&format!("Mongo doc not found for {}", id));
                Ok(None)
            }
        }
    }

    async fn find_by_key(&self, key: MetadataKey) -> Result<Option<Metadata>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [FIND BY KEY] {:?}: {:?} ",
            key.kind(), key
        ));

        let id = key.mongo_id();
        let doc_opt = self.metadata_collection.find_one(doc! { "_id": id.clone().as_str() }).await?;

        // Ok(doc_opt.map(|d| d.meta))
        match doc_opt {
            Some(d) => {
                timer.log();
                Ok(Some(d.meta))
            },
            None => {
                timer.error_with_message(&format!("Mongo doc not found for {}", id));
                Ok(None)
            }
        }
    }

    async fn find_all(&self) -> Result<Vec<Metadata>, Error> {
        let timer = TimePrinter::with_message("[REPOSITORY] [META DATA] [FIND ALL]");

        let mut cursor = self.metadata_collection.find(doc! {}).await?;

        let mut out = Vec::new();
        while let Some(item) = cursor.next().await {
            out.push(item?);
        }
        timer.log();
        Ok(out.into_iter().map(|d| d.meta).collect())
    }

    async fn find_all_by_type(&self, metadata_type: &str) -> Result<Vec<Metadata>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [META DATA] [FIND BY TYPE] type: {:?} ",
            metadata_type
        ));

        let mut cursor = self.metadata_collection
            .find(doc! { "type": metadata_type })
            .await?;

        let mut out = Vec::new();
        while let Some(item) = cursor.next().await {
            out.push(item?);
        }
        timer.log();
        Ok(out.into_iter().map(|d| d.meta).collect())
    }
}

