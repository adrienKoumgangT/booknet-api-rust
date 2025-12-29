use std::collections::HashMap;
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc, FixedOffset};
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{Bson, doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Database, Collection,
};
use mongodb::bson::to_document;
use neo4rs::{query, Graph, Query, Txn};

use crate::model::book_model::BookEmbed;
use crate::model::review_model::Review;
use crate::model::user_model::{ReaderNode, User, UserEmbed, UserPreference};
use crate::shared::constant::LIMIT_DEFAULT;
use crate::shared::logging::log::TimePrinter;


#[async_trait]
pub trait UserRepositoryInterface {
    async fn insert(&self, user: User) -> Result<String, Error>;
    async fn insert_many(&self, users: Vec<User>) -> Result<Vec<String>, Error>;
    async fn update_name(&self, user_id: &str, name: &str) -> Result<bool, Error>;
    async fn update_password(&self, user_id: &str, password: &str) -> Result<bool, Error>;
    async fn update_image_url(&self, user_id: &str, image_url: &str) -> Result<bool, Error>;
    async fn update_preference(&self, user_id: &str, preference: UserPreference) -> Result<bool, Error>;
    async fn update_shelf(&self, user_id: &str, shelf: Vec<BookEmbed>) -> Result<bool, Error>;
    async fn add_book_to_shelf(&self, user_id: &str, book: BookEmbed) -> Result<bool, Error>;
    async fn remove_book_from_shelf(&self, user_id: &str, book_id: &str) -> Result<bool, Error>;
    async fn update_reviews(&self, user_id: &str, reviews: Vec<String>) -> Result<bool, Error>;
    async fn add_review(&self, user_id: &str, review: Review) -> Result<bool, Error>;
    async fn remove_review(&self, user_id: &str, review: Review) -> Result<bool, Error>;
    async fn delete(&self, user_id: &str) -> Result<bool, Error>;
    async fn delete_many(&self, user_ids: Vec<&str>) -> Result<bool, Error>;
    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
    async fn find_all(&self, page: Option<u64>, limit: Option<u64>) -> Result<Vec<User>, Error>;
}

#[derive(Clone)]
pub struct UserRepository {
    pub mongo_client: Client,
    pub user_collection: Collection<User>,
    pub neo4j_client: Graph,
}

impl UserRepository {
    pub fn new(mongo_client: Client, mongo_database: Database, neo4j_client: Graph) -> Self {
        let user_collection = mongo_database.collection::<User>("users");
        UserRepository {
            mongo_client,
            user_collection,
            neo4j_client,
        }
    }
}


#[async_trait]
impl UserRepositoryInterface for UserRepository {
    async fn insert(&self, user: User) -> Result<String, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [INSERT] data: {:?} ",
            user
        ));

        if(user.role.save_in_noe4j()) {
            let mut mongo_session = self.mongo_client.start_session().await?;
            mongo_session.start_transaction().await?;

            let result_insert = self.user_collection
                .insert_one(user.clone())
                .session(&mut mongo_session)
                .await;

            match result_insert {
                Ok(result_insert) => {
                    let mut neo4j_tx = self.neo4j_client.start_txn().await?;
                    
                    let mut reader_node = ReaderNode::from(&user);
                    reader_node.user_id = result_insert.inserted_id.to_string();
                    
                    let query = query("CREATE (r:Reader {user_id:$user_id, name:$name})")
                        // .param("id", reader_node.id.unwrap())
                        .param("user_id", reader_node.user_id.as_str())
                        .param("name", reader_node.name.as_str());
                    let result = neo4j_tx.run(query).await;
                    
                    match result {
                        Ok(_) => {
                            mongo_session.commit_transaction().await?;
                            neo4j_tx.commit().await?;
                            timer.log();
                            Ok(result_insert.inserted_id.to_string())
                        }
                        Err(e) => {
                            let _ = mongo_session.abort_transaction().await;
                            let _ = neo4j_tx.rollback().await;
                            timer.error_with_message(&format!("Error adding user: {}", e));
                            Err(e.into())
                        }
                    }
                },
                Err(e) => {
                    let _ = mongo_session.abort_transaction().await;
                    timer.error_with_message(&format!("Error adding user: {}", e));
                    Err(e.into())
                }
            }
        } else {
            let result_insert = self.user_collection.insert_one(user).await;
            match result_insert {
                Ok(result_insert) => {
                    timer.log();
                    Ok(result_insert.inserted_id.as_str().unwrap().parse()?)
                },
                Err(e) => Err(e.into()),
            }
        }
    }

    async fn insert_many(&self, users: Vec<User>) -> Result<Vec<String>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [INSERT MANY] count: {:?} ",
            users.len()
        ));

        if users.is_empty() {
            return Ok(vec![]);
        }

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let result_insert = self.user_collection
            .insert_many(&users)
            .session(&mut mongo_session)
            .await;

        match result_insert {
            Ok(result_insert) => {
                let mut neo4j_rows: Vec<HashMap<String, String>> = Vec::with_capacity(users.len());
                let mut success_ids: Vec<String> = Vec::with_capacity(users.len());

                for(i, user) in users.iter().enumerate() {
                    if user.role.save_in_noe4j() {
                        if let Some(id_bson) = result_insert.inserted_ids.get(&i) {
                            if let Bson::ObjectId(oid) = id_bson {
                                let id_str = oid.to_string();
                                success_ids.push(id_str.clone());

                                let mut row = HashMap::new();
                                row.insert("user_id".to_string(), id_str);
                                row.insert("name".to_string(), user.name.clone());

                                neo4j_rows.push(row);
                            }
                        }
                    }
                }

                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                let cypher = "
                UNWIND $rows AS row
                CREATE (r:Reader {user_id: row.user_id, name: row.name})
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
                        timer.error_with_message(&format!("Error adding users: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                let _ = mongo_session.abort_transaction().await;
                timer.error_with_message(&format!("Error adding users: {}", e));
                Err(e.into())
            }
        }
    }

    async fn update_name(&self, user_id: &str, name: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE NAME] user_id: {:?} name: {:?} ",
            user_id, name
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let mut mongo_session = self.mongo_client.start_session().await?;
                mongo_session.start_transaction().await?;
                
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "name": name } };

                let result_update = self.user_collection.
                    update_one(filter, update)
                    .session(&mut mongo_session)
                    .await;
                
                match result_update {
                    Ok(result_update) => {
                        let mut neo4j_tx = self.neo4j_client.start_txn().await?;
                        
                        let query = query("MATCH (r:Reader {user_id:$user_id}) SET r.name=$name")
                            .param("user_id", id.to_string())
                            .param("name", name);
                        let result = neo4j_tx.run(query).await;
                        
                        match result {
                            Ok(_) => {
                                mongo_session.commit_transaction().await?;
                                neo4j_tx.commit().await?;
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            }
                            Err(e) => {
                                let _ = mongo_session.abort_transaction().await;
                                let _ = neo4j_tx.rollback().await;
                                timer.error_with_message(&format!("Error updating user: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn update_password(&self, user_id: &str, password: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE PASSWORD] user_id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "password": password } };

                let result_update = self.user_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            }
            Err(e) => {
                timer.error_with_message(&format!("Invalid user id: {}", e));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn update_image_url(&self, user_id: &str, image_url: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE IMAGE] user_id: {:?} image url: {:?} ",
            user_id, image_url
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "image_url": image_url } };

                let result_update = self.user_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn update_preference(&self, user_id: &str, preference: UserPreference) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE PREFERENCE] user_id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let pref_doc = to_document(&preference)?;
                let update = doc! { "$set": { "preference": pref_doc } };

                let result_update = self.user_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn update_shelf(&self, user_id: &str, shelf: Vec<BookEmbed>) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE SHELF] user_id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let shelf_doc = to_document(&shelf)?;
                let update = doc! { "$set": { "shelf": shelf_doc } };

                let result_update = self.user_collection.update_one(filter, update).await;
                match result_update {
                    Ok(result_update) => {
                        timer.log();
                        Ok(result_update.modified_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn add_book_to_shelf(&self, user_id: &str, book: BookEmbed) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [ADD BOOK TO SHELF] user_id: {:?} book_id: {:?} ",
            user_id, book.book_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let mut mongo_session = self.mongo_client.start_session().await?;
                mongo_session.start_transaction().await?;

                let filter = doc! {"_id": &id };
                let book_doc = to_document(&book)?;
                let update = doc! { "$push": { "shelf": book_doc } };

                let result_update = self.user_collection
                    .update_one(filter, update)
                    .session(&mut mongo_session)
                    .await;

                match result_update {
                    Ok(result_update) => {
                        let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                        let cypher = "
                            MATCH (r:Reader {mid: $user_id})

                            // 1. Ensure the Book exists in the Graph
                            // We merge on 'mid' (Mongo ID) to avoid duplicates
                            MERGE (b:Book {mid: $book_id})

                            // 2. If the book is new to Neo4j, initialize its properties
                            ON CREATE SET
                                b.id = randomUUID(),  // Generate a Neo4j-specific UUID
                                b.title = $book_title

                            // 3. Create the Shelf Relationship
                            MERGE (r)-[rel:ADDED_TO_SHELF]->(b)

                            // 4. Update Relationship Properties
                            SET rel.ts = datetime(),
                            // Default to 'WANT_TO_READ' if status is missing,
                            // otherwise keep the existing status
                            rel.status = COALESCE(rel.status, 'ADDED')
                        ";
                        let query = query(cypher)
                            .param("user_id", user_id)
                            .param("book_id", book.book_id.to_string())
                            .param("book_title", book.title);
                        let result = neo4j_tx.run(query).await;

                        match result {
                            Ok(_) => {
                                mongo_session.commit_transaction().await?;
                                neo4j_tx.commit().await?;
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            },
                            Err(e) => {
                                mongo_session.abort_transaction().await?;
                                neo4j_tx.rollback().await?;
                                timer.error_with_message(&format!("Error updating user: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn remove_book_from_shelf(&self, user_id: &str, book_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [REMOVE BOOK FROM SHELF] user_id: {:?} book_id: {:?} ",
            user_id, book_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let book_oid = ObjectId::parse_str(book_id);
                match book_oid {
                    Ok(book_oid) => {
                        let mut mongo_session = self.mongo_client.start_session().await?;
                        mongo_session.start_transaction().await?;

                        let filter = doc! {"_id": &id };
                        let book_id_filter = doc! {"book_id": book_oid };
                        let update = doc! { "$pull": { "shelf": book_id_filter } };

                        let result_update = self.user_collection
                            .update_one(filter, update)
                            .session(&mut mongo_session)
                            .await;

                        match result_update {
                            Ok(result_update) => {
                                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                                let cypher = "
                                MATCH (r:Reader {mid: $user_id})-[rel:ADDED_TO_SHELF]->(b:Book {mid: $book_id})
                                DELETE rel
                                ";
                                let query = query(cypher)
                                    .param("user_id", user_id)
                                    .param("book_id", book_oid.to_string());
                                let result = neo4j_tx.run(query).await;

                                match result {
                                    Ok(_) => {
                                        mongo_session.commit_transaction().await?;
                                        neo4j_tx.commit().await?;
                                        timer.log();
                                        Ok(result_update.modified_count > 0)
                                    },
                                    Err(e) => {
                                        mongo_session.abort_transaction().await?;
                                        neo4j_tx.rollback().await?;
                                        timer.error_with_message(&format!("Error removing book from user shelf: {}", e));
                                        Err(e.into())
                                    },
                                }
                            },
                            Err(e) => {
                                timer.error_with_message(&format!("Error updating user: {}", e));
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
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn update_reviews(&self, user_id: &str, reviews: Vec<String>) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE REVIEWS] user_id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let reviews_ois = reviews.iter().map(|review_id| ObjectId::parse_str(review_id)).collect::<Result<Vec<ObjectId>, _>>();
                match reviews_ois {
                    Ok(reviews_ois) => {
                        let reviews_doc = to_document(&reviews_ois)?;
                        let update = doc! { "$set": { "reviews": reviews_doc } };

                        let result_update = self.user_collection.update_one(filter, update).await;
                        match result_update {
                            Ok(result_update) => {
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            },
                            Err(e) => {
                                timer.error_with_message(&format!("Error updating user: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(_) => {
                        timer.error_with_message(&"Invalid review id".to_string());
                        Err(anyhow!("Invalid review id"))
                    }
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn add_review(&self, user_id: &str, review: Review) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [ADD REVIEW] user_id: {:?} review: {:?} ",
            user_id, review
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let mut mongo_session = self.mongo_client.start_session().await?;
                mongo_session.start_transaction().await?;

                let review_id = review.id.unwrap();
                let filter = doc! {"_id": &id };
                let update = doc! { "$push": { "reviews": review_id } };

                let result_update = self.user_collection
                    .update_one(filter, update)
                    .session(&mut mongo_session)
                    .await;

                match result_update {
                    Ok(result_update) => {
                        let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                        let ts_param = review.date_added.unwrap().timestamp_millis();
                        let cypher = "
                            MATCH (u:Reader {user_id: $user_id})
                            MATCH (b:Book {book_id: $book_id})
                            MERGE (u)-[r:RATED]->(b)
                            SET r.rating = $rating, r.ts = $ts
                        ";
                        let query = query(cypher)
                            .param("user_id", user_id)
                            .param("book_id", review.book_id.to_string())
                            .param("rating", review.score)
                            .param("ts", ts_param);
                        let result = neo4j_tx.run(query).await;

                        match result {
                            Ok(_) => {
                                mongo_session.commit_transaction().await?;
                                neo4j_tx.commit().await?;
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            }
                            Err(e) => {
                                mongo_session.abort_transaction().await?;
                                neo4j_tx.rollback().await?;
                                timer.error_with_message(&format!("Error adding review to user: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error updating user: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn remove_review(&self, user_id: &str, review: Review) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [REMOVE REVIEW] user_id: {:?} review_id: {:?} ",
            user_id, review.id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(user_oid) => {
                let mut mongo_session = self.mongo_client.start_session().await?;
                mongo_session.start_transaction().await?;

                let review_id = review.id.unwrap();
                let filter = doc! {"_id": user_oid };
                let update = doc! { "$pull": { "reviews": review_id } };

                let result_update = self.user_collection
                    .update_one(filter, update)
                    .session(&mut mongo_session)
                    .await;

                match result_update {
                    Ok(result_update) => {
                        let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                        let cypher = "
                            MATCH (u:Reader {user_id: $user_id})-[r:RATED]->(b:Book {book_id: $book_id})
                            DELETE r
                        ";

                        let query = query(cypher)
                            .param("user_id", user_id)
                            .param("book_id", review.book_id.to_string());

                        let result = neo4j_tx.run(query).await;

                        match result {
                            Ok(_) => {
                                mongo_session.commit_transaction().await?;
                                neo4j_tx.commit().await?;
                                timer.log();
                                Ok(result_update.modified_count > 0)
                            }
                            Err(e) => {
                                mongo_session.abort_transaction().await?;
                                neo4j_tx.rollback().await?;
                                timer.error_with_message(&format!("Error deleting review from Neo4j: {}", e));
                                Err(e.into())
                            }
                        }
                    },
                    Err(e) => {
                        mongo_session.abort_transaction().await?;
                        timer.error_with_message(&format!("Error updating user in Mongo: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn delete(&self, user_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [DELETE] id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let result_delete = self.user_collection.delete_one(filter).await;
                match result_delete {
                    Ok(result_delete) => {
                        timer.log();
                        Ok(result_delete.deleted_count > 0)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error deleting user: {}", e));
                        Err(e.into())
                    },
                }
            },
            Err(_) => {
                timer.error_with_message(&format!("Invalid user id: {}", user_id));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn delete_many(&self, user_ids: Vec<&str>) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [DELETE MANY] ids: {:?} ",
            user_ids
        ));

        let ids: Vec<_> = user_ids
            .iter()
            .filter_map(|&id| ObjectId::parse_str(id).ok())
            .collect();

        if ids.is_empty() {
            return Ok(true);
        }

        let neo4j_ids: Vec<String> = ids.iter().map(|oid| oid.to_string()).collect();

        let mut mongo_session = self.mongo_client.start_session().await?;
        mongo_session.start_transaction().await?;

        let filter = doc! {"_id": { "$in": ids } };
        let result_delete = self.user_collection
            .delete_many(filter)
            .session(&mut mongo_session)
            .await;
        match result_delete {
            Ok(result_delete) => {
                let mut neo4j_tx = self.neo4j_client.start_txn().await?;

                let query = query("OPTIONAL MATCH (r:Reader) WHERE r.user_id IN user_ids DETACH DELETE r")
                    .param("user_ids", neo4j_ids);
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
                        timer.error_with_message(&format!("Error deleting users: {}", e));
                        Err(e.into())
                    }
                }
            },
            Err(e) => {
                mongo_session.abort_transaction().await?;
                timer.error_with_message(&format!("Error deleting users: {}", e));
                Err(e.into())
            }
        }
    }

    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [FIND BY ID] id: {:?} ",
            user_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let result = self.user_collection.find_one(filter).await;
                match result {
                    Ok(result) => {
                        timer.log();
                        Ok(result)
                    },
                    Err(e) => {
                        timer.error_with_message(&format!("Error finding user: {}", e));
                        Err(e.into())
                    },
                }
            }
            Err(e) => {
                timer.error_with_message(&format!("Invalid user id: {}", e));
                Err(anyhow!("Invalid user id"))
            }
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [FIND BY USERNAME] username: {:?} ",
            username
        ));

        let filter = doc! { "username": username };
        let result_find = self.user_collection.find_one(filter).await;
        match result_find {
            Ok(result_find) => {
                timer.log();
                Ok(result_find)
            },
            Err(e) => {
                timer.error_with_message(&format!("Error finding user: {}", e));
                Err(e.into())
            },
        }
    }

    async fn find_all(&self, page: Option<u64>, limit: Option<u64>) -> Result<Vec<User>, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [FIND ALL] page: {:?}, limit: {:?}",
            page, limit
        ));

        let skip = page.unwrap_or(0) * limit.unwrap_or(LIMIT_DEFAULT);

        let filter = doc! {};
        let result_find = self.user_collection
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
                timer.error_with_message(&format!("Error finding users: {}", e));
                Err(e.into())
            },
        }
    }
}

