use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::User, utils::constants::USERS_COLLECTION};

#[derive(Clone)]
pub struct UsersRepository {
    collection: Collection<User>,
}

impl UsersRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(USERS_COLLECTION),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<User>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, user_id: &str) -> Result<Option<User>> {
        self.collection.find_one(doc! { "userId": user_id }).await
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<User>> {
        self.collection.find_one(doc! { "token": token }).await
    }

    pub async fn insert(&self, user: &User) -> Result<InsertOneResult> {
        self.collection.insert_one(user).await
    }

    pub async fn update(&self, user: &User) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "userId": &user.user_id },
                doc! { "$set": serialize_to_document(user)? },
            )
            .await
    }

    pub async fn delete(&self, user_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "userId": user_id }).await
    }
}
