use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, oid::ObjectId},
    results::{DeleteResult, InsertOneResult, UpdateResult},
    Client, Database, Collection,
};
use mongodb::bson::to_document;
use neo4rs::{query, Graph, Query, Txn};

use crate::model::book_model::BookEmbed;
use crate::model::user_model::{ReaderNode, User, UserEmbed, UserPreference};
use crate::shared::logging::log::TimePrinter;


#[async_trait]
pub trait UserRepositoryInterface {
    async fn insert(&self, user: User) -> Result<String, Error>;
    async fn update_name(&self, user_id: &str, name: &str) -> Result<bool, Error>;
    async fn update_password(&self, user_id: &str, password: &str) -> Result<bool, Error>;
    async fn update_image_url(&self, user_id: &str, image_url: &str) -> Result<bool, Error>;
    async fn update_preference(&self, user_id: &str, preference: UserPreference) -> Result<bool, Error>;
    async fn update_shelf(&self, user_id: &str, shelf: Vec<UserEmbed>) -> Result<bool, Error>;
    async fn add_book_to_shelf(&self, user_id: &str, book: BookEmbed) -> Result<bool, Error>;
    async fn remove_book_from_shelf(&self, user_id: &str, book_id: &str) -> Result<bool, Error>;
    async fn update_reviews(&self, user_id: &str, reviews: Vec<String>) -> Result<bool, Error>;
    async fn add_review(&self, user_id: &str, review_id: &str) -> Result<bool, Error>;
    async fn remove_review(&self, user_id: &str, review_id: &str) -> Result<bool, Error>;
    async fn delete(&self, user_id: &str) -> Result<bool, Error>;
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
            let mut neo4j_tx = self.neo4j_client.start_txn().await?;
            let reader_node = ReaderNode::from(&user);
            let query = query("CREATE (r:Reader {user_id:user_id, name:$name})")
                // .param("id", reader_node.id.unwrap())
                .param("user_id", reader_node.user_id.as_str())
                .param("name", reader_node.name.as_str());
            neo4j_tx.run(query).await?;

            let mut mongo_session = self.mongo_client.start_session().await?;
            mongo_session.start_transaction().await?;

            let result_insert = self.user_collection
                .insert_one(user.clone())
                .session(&mut mongo_session)
                .await;

            match result_insert {
                Ok(result_insert) => {
                    mongo_session.commit_transaction().await?;

                    if let Err(e) = neo4j_tx.commit().await {
                        let _ = self.user_collection.delete_one(doc! { "_id": &result_insert.inserted_id }).await;
                        timer.error_with_message(&format!("Error adding user: {}", e));
                        return Err(e.into());
                    }

                    timer.log();
                    Ok(result_insert.inserted_id.as_str().unwrap().parse()?)
                },
                Err(e) => {
                    let _ = mongo_session.abort_transaction().await;
                    let _ = neo4j_tx.rollback().await;
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

    async fn update_name(&self, user_id: &str, name: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [UPDATE NAME] user_id: {:?} name: {:?} ",
            user_id, name
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let filter = doc! {"_id": &id };
                let update = doc! { "$set": { "name": name } };

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

    async fn update_shelf(&self, user_id: &str, shelf: Vec<UserEmbed>) -> Result<bool, Error> {
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
                let filter = doc! {"_id": &id };
                let book_doc = to_document(&book)?;
                let update = doc! { "$push": { "shelf": book_doc } };

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
                return Err(anyhow!("Invalid user id"))
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
                let book_oi = ObjectId::parse_str(book_id);
                match book_oi {
                    Ok(book_id) => {
                        let filter = doc! {"_id": &id };
                        let book_id_filter = doc! {"book_id": book_id };
                        let update = doc! { "$pull": { "shelf": book_id_filter } };
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

    async fn add_review(&self, user_id: &str, review_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [ADD REVIEW] user_id: {:?} review_id: {:?} ",
            user_id, review_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let review_oi = ObjectId::parse_str(review_id);
                match review_oi {
                    Ok(review_oi) => {
                        let filter = doc! {"_id": &id };
                        let review_id_filter = doc! {"review_id": review_oi };
                        let update = doc! { "$push": { "reviews": review_id_filter } };

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
                        timer.error_with_message(&format!("Invalid review id: {}", review_id));
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

    async fn remove_review(&self, user_id: &str, review_id: &str) -> Result<bool, Error> {
        let timer = TimePrinter::with_message(&format!(
            "[REPOSITORY] [USER] [REMOVE REVIEW] user_id: {:?} review_id: {:?} ",
            user_id, review_id
        ));

        let id = ObjectId::parse_str(user_id);
        match id {
            Ok(id) => {
                let review_oi = ObjectId::parse_str(review_id);
                match review_oi {
                    Ok(review_oi) => {
                        let filter = doc! {"_id": &id };
                        let update = doc! { "$pull": { "reviews": review_oi } };

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
                        timer.error_with_message(&format!("Invalid review id: {}", review_id));
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

        let skip = page.unwrap_or(0) * limit.unwrap_or(10);

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

