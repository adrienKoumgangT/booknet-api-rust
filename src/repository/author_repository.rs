use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{Bson, doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Database, Collection,
};
use mongodb::bson::to_document;
use neo4rs::{query, Graph, Query, Txn};
use std::collections::HashMap;
use crate::model::author_model::{Author, AuthorNode};
use crate::model::book_model::BookEmbed;
use crate::shared::constant::LIMIT_DEFAULT;
use crate::shared::logging::log::TimePrinter;

#[async_trait]
pub trait AuthorRepositoryInterface {
    async fn insert(&self, author: Author) -> Result<String, Error>;
    async fn insert_many(&self, authors: Vec<Author>) -> Result<Vec<String>, Error>;
    async fn update_description(&self, author_id: &str, description: &str) -> Result<bool, Error>;
    async fn update_image_url(&self, author_id: &str, image_url: &str) -> Result<bool, Error>;
    async fn add_book(&self, author_id: &str, book_embed: BookEmbed) -> Result<bool, Error>;
    async fn remove_book(&self, author_id: &str, book_id: &str) -> Result<bool, Error>;
    async fn delete(&self, author_id: &str) -> Result<bool, Error>;
    async fn delete_many(&self, author_ids: Vec<&str>) -> Result<bool, Error>;
    async fn find_by_id(&self, author_id: &str) -> Result<Option<Author>, Error>;
    async fn find_by_ids(&self, author_ids: Vec<&str>) -> Result<Vec<Author>, Error>;
    async fn find_by_object_ids(&self, author_object_ids: Vec<ObjectId>) -> Result<Vec<Author>, Error>;
    async fn find_all(&self, page: Option<u64>, limit: Option<u64>) -> Result<Vec<Author>, Error>;
}

#[derive(Clone)]
pub struct AuthorRepository {
    pub mongo_client: Client,
    pub author_collection: Collection<Author>,
    pub neo4j_client: Graph,
}

impl AuthorRepository {
    pub fn new(mongo_client: Client, mongo_database: Database, neo4j_client: Graph) -> Self {
        let author_collection = mongo_database.collection::<Author>("authors");
        AuthorRepository {
            mongo_client,
            author_collection,
            neo4j_client,
        }
    }
}


#[async_trait]
impl AuthorRepositoryInterface for AuthorRepository {
    async fn insert(&self, author: Author) -> Result<String, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [INSERT] data: {:?}",
            author
        ));

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let result_insert = self.author_collection
            .insert_one(&author)
            .session(&mut mongo_session)
            .await;

        match result_insert {
            Ok(result_insert) => {
                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                let mut author_node = AuthorNode::from(&author);
                author_node.author_id = result_insert.inserted_id.to_string();

                let query = query("CREATE (a:Author {author_id:$author_id, name:$name})")
                    .param("author_id", author_node.author_id.as_str())
                    .param("name", author_node.name.as_str());
                let result = neo4j_tx.run(query).await;

                match result {
                    Ok(_) => {
                        mongo_session.commit_transaction().await?;
                        neo4j_tx.commit().await?;
                        timer.log();
                        Ok(result_insert.inserted_id.to_string())
                    },
                    Err(e) => {
                        let _ = mongo_session.abort_transaction().await;
                        let _ = neo4j_tx.rollback().await;
                        timer.error_with_message(&format!("Error adding author to Neo4j: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                let _ = mongo_session.abort_transaction().await;
                timer.error_with_message(&format!("Error adding author: {}", e));
                Err(e.into())
            }
        }
    }

    async fn insert_many(&self, authors: Vec<Author>) -> Result<Vec<String>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [INSERT MULTI] count: {}",
            authors.len()
        ));

        if authors.is_empty() {
            return Ok(vec![]);
        }

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let result_insert = self.author_collection
            .insert_many(&authors) // Pass options if needed
            .session(&mut mongo_session)
            .await;

        match result_insert {
            Ok(result_insert) => {
                let mut neo4j_rows: Vec<HashMap<String, String>> = Vec::with_capacity(authors.len());
                let mut success_ids: Vec<String> = Vec::with_capacity(authors.len());

                for (i, author) in authors.iter().enumerate() {
                    // Get the ID generated by Mongo for this specific index
                    if let Some(id_bson) = result_insert.inserted_ids.get(&i) {
                        if let Bson::ObjectId(oid) = id_bson {
                            let id_str = oid.to_string();
                            success_ids.push(id_str.clone());

                            // Prepare the row for Neo4j
                            // We create a map for each author to send as a batch parameter
                            let mut row = HashMap::new();
                            row.insert("author_id".to_string(), id_str);
                            row.insert("name".to_string(), author.name.clone());

                            neo4j_rows.push(row);
                        }
                    }
                }

                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                // We use UNWIND to unpack the list of maps and create nodes in one go
                let cypher = "
                UNWIND $rows AS row
                CREATE (a:Author {author_id: row.author_id, name: row.name})
                ";

                let query = query(cypher).param("rows", neo4j_rows);
                let result = neo4j_tx.run(query).await;

                match result {
                    Ok(_) => {
                        mongo_session.commit_transaction().await?;
                        neo4j_tx.commit().await?;
                        timer.log();
                        Ok(success_ids)
                    },
                    Err(e) => {
                        let _ = mongo_session.abort_transaction().await;
                        let _ = neo4j_tx.rollback().await;
                        timer.error_with_message(&format!("Error adding authors to Neo4j: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                let _ = mongo_session.abort_transaction().await;
                timer.error_with_message(&format!("Error adding authors to Mongo: {}", e));
                Err(e.into())
            }
        }
    }

    async fn update_description(&self, author_id: &str, description: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [UPDATE DESCRIPTION] author_id: {:?} description: {:?}",
            author_id, description
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "description": description } };

                let result_update = self.author_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating author: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid author id: {}", author_id));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn update_image_url(&self, author_id: &str, image_url: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [UPDATE IMAGE] author_id: {:?} image_url: {:?}",
            author_id, image_url
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "image_url": image_url } };

                let result_update = self.author_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating author: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid author id: {}", author_id));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn add_book(&self, author_id: &str, book: BookEmbed) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [ADD BOOK] author_id: {:?} book_embed: {:?}",
            author_id, book.book_id
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let book_doc = to_document(&book)?;
                let update = doc! { "$push": { "books": book_doc } };

                let result_update = self.author_collection
                    .update_one(filter, update)
                    .await;

                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error adding book to author: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid author id: {}", author_id));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn remove_book(&self, author_id: &str, book_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [REMOVE BOOK] author_id: {:?} book_id: {:?}",
            author_id, book_id
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let book_oid = ObjectId::parse_str(book_id);
                match book_oid {
                    Ok(book_oid) => {
                        let filter = doc! {"_id": &id };
                        let book_filter = doc! {"book_id": book_oid };
                        let update = doc! { "$pull": { "books": book_filter } };

                        let result_update = self.author_collection
                            .update_one(filter, update)
                            .await;

                        match result_update {
                            Ok(result_update) => {
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            },
                            Err(e) => {
                                timer.error_with_message(&format!("Error removing book from author: {}", e));
                                Err(e.into())
                            },
                        }
                    },
                    Err(_) => {
                        timer.error_with_message(&format!("Invalid book id: {}", book_id));
                        Err(anyhow!("Invalid book id"))
                    }
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid author id: {}", author_id));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn delete(&self, author_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [DELETE] author_id: {:?}",
            author_id
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let mut mongo_session = self.mongo_client.start_session().await?;
                mongo_session.start_transaction().await?;

                let filter = doc! {"_id": &id };
                let result_delete = self.author_collection
                    .delete_one(filter)
                    .session(&mut mongo_session)
                    .await;

                match result_delete {
                    Ok(result_delete) => {
                        let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                        let query = query("MATCH (a:Author {author_id:$author_id}) DETACH DELETE a")
                            .param("author_id", author_id);
                        let result = neo4j_tx.run(query).await;

                        match result {
                            Ok(_) => {
                                mongo_session.commit_transaction().await?;
                                neo4j_tx.commit().await?;
                                timer.log();
                                Ok(result_delete.deleted_count > 0)
                            }
                            Err(e) => {
                                mongo_session.abort_transaction().await?;
                                neo4j_tx.rollback().await?;
                                timer.error_with_message(&format!("Error deleting author: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error deleting author: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid author id: {}", author_id));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn delete_many(&self, author_ids: Vec<&str>) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [DELETE MULTI] author_ids: {:?}",
            author_ids
        ));

        let ids: Vec<_> = author_ids
            .iter()
            .filter_map(|&id| ObjectId::parse_str(id).ok())
            .collect();

        if ids.is_empty() {
            return Ok(true);
        }

        let neo4j_ids: Vec<String> = ids.iter().map(|oid| oid.to_string()).collect();

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let filter = doc! {"_id": {"$in": ids }};
        let result_delete = self.author_collection
            .delete_many(filter)
            .session(&mut mongo_session)
            .await;
        match result_delete {
            Ok(result_delete) => {
                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                let query = query("OPTIONAL MATCH (a:Author) WHERE a.author_id IN $author_ids DETACH DELETE a")
                    .param("author_ids", neo4j_ids);
                let result = neo4j_tx.run(query).await;

                match result {
                    Ok(_) => {
                        mongo_session.commit_transaction().await?;
                        neo4j_tx.commit().await?;
                        timer.log();
                        Ok(result_delete.deleted_count > 0)
                    },
                    Err(e) => {
                        mongo_session.abort_transaction().await?;
                        neo4j_tx.rollback().await?;
                        timer.error_with_message(&format!("Error deleting authors: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                mongo_session.abort_transaction().await?;
                timer.error_with_message(&format!("Error deleting authors: {}", e));
                Err(e.into())
            }
        }
    }

    async fn find_by_id(&self, author_id: &str) -> Result<Option<Author>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [FIND BY ID] author_id: {:?}",
            author_id
        ));

        let id = ObjectId::parse_str(author_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let result = self.author_collection.find_one(filter).await;
                match result {
                    Ok(result) => {
                        timer.log();
                        Ok(result)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error finding author: {}", e));
                        Err(e.into())
                    },
                }
            }
            Err(e) => {
                timer.error_with_message(&format!("Invalid author id: {}", e));
                Err(anyhow!("Invalid author id"))
            }
        }
    }

    async fn find_by_ids(&self, author_ids: Vec<&str>) -> Result<Vec<Author>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [FIND BY IDS] author_ids: {:?}",
            author_ids
        ));

        let ids: Vec<_> = author_ids
            .iter()
            .filter_map(|&id| ObjectId::parse_str(id).ok())
            .collect();

        let filter = doc! {"_id": {"$in": ids }};
        let result_find = self.author_collection.find(filter).await;
        match result_find {
            Ok(result_find) => {
                timer.log();
                Ok(result_find.try_collect().await?)
            },
            Err(e) => {
                timer.error_with_message(&format!("Error finding authors: {}", e));
                Err(e.into())
            },
        }
    }

    async fn find_by_object_ids(&self, author_object_ids: Vec<ObjectId>) -> Result<Vec<Author>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [FIND BY OBJECT IDS] author_object_ids: {:?}",
            author_object_ids
        ));

        let filter = doc! {"_id": {"$in": author_object_ids }};
        let result_find = self.author_collection.find(filter).await;
        match result_find {
            Ok(result_find) => {
                timer.log();
                Ok(result_find.try_collect().await?)
            },
            Err(e) => {
                timer.error_with_message(&format!("Error finding authors: {}", e));
                Err(e.into())
            },
        }
    }

    async fn find_all(&self, page: Option<u64>, limit: Option<u64>) -> Result<Vec<Author>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [AUTHOR] [FIND ALL] page: {:?} limit: {:?}",
            page, limit
        ));

        let skip = page.unwrap_or(0) * limit.unwrap_or(LIMIT_DEFAULT);

        let filter = doc! {};
        let result_find = self.author_collection
            .find(filter)
            .skip(skip)
            .limit(limit.unwrap_or(10) as i64)
            .await;

        match result_find {
            Ok(result_find) => {
                timer.log();
                Ok(result_find.try_collect().await?)
            },
            Err(e) => {
                timer.error_with_message(&format!("Error finding authors: {}", e));
                Err(e.into())
            },
        }
    }
}
