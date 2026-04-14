use std::collections::{HashMap, HashSet};

use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::User, utils::constants::USERS_COLLECTION};

use super::common::{CollectionSpec, UpdateBuilder, ensure_collection};

#[derive(Clone, Default)]
pub struct UserUpdate {
    builder: UpdateBuilder,
}

impl UserUpdate {
    pub fn with_avatar(mut self, avatar: Option<String>) -> Self {
        self.builder = self.builder.set(doc! { "avatar": avatar });
        self
    }

    pub fn with_avatar_decoration(mut self, avatar_decoration: Option<String>) -> Self {
        self.builder = self
            .builder
            .set(doc! { "avatarDecoration": avatar_decoration });
        self
    }

    pub fn with_bots_limit(mut self, bots_limit: i32) -> Self {
        self.builder = self.builder.set(doc! { "botsLimit": bots_limit });
        self
    }

    pub fn with_mail(mut self, mail: String) -> Self {
        self.builder = self.builder.set(doc! { "mail": mail });
        self
    }

    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.builder = self.builder.set(doc! { "suspended": suspended });
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.builder = self.builder.set(doc! { "username": username });
        self
    }

    pub fn build(self) -> Document {
        self.builder.build()
    }
}

#[derive(Clone)]
pub struct UsersRepository {
    collection: Collection<User>,
}

impl UsersRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, USERS_COLLECTION, CollectionSpec::Standard).await?,
        })
    }

    pub async fn find_all(&self) -> Result<Vec<User>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, user_id: &str) -> Result<Option<User>> {
        self.collection.find_one(doc! { "userId": user_id }).await
    }

    pub async fn find_many_by_ids(
        &self,
        user_ids: &HashSet<String>,
    ) -> Result<HashMap<String, User>> {
        let cursor = self
            .collection
            .find(doc! { "userId": { "$in": user_ids } })
            .await?;
        let users = cursor.try_collect::<Vec<_>>().await?;
        Ok(users
            .into_iter()
            .map(|user| (user.user_id.clone(), user))
            .collect())
    }

    pub async fn insert(&self, user: &User) -> Result<InsertOneResult> {
        self.collection.insert_one(user).await
    }

    pub async fn update(&self, user_id: &str, updated_user: UserUpdate) -> Result<Option<User>> {
        let updates = updated_user.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "userId": user_id }, doc! { "$set": updates })
            .with_options(options)
            .await
    }

    pub async fn delete_by_id(&self, user_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "userId": user_id }).await
    }
}
