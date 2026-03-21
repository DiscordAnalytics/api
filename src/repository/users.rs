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

#[derive(Clone, Default)]
pub struct UserUpdate {
    updates: Document,
}

impl UserUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_avatar(mut self, avatar: Option<String>) -> Self {
        self.updates.insert("avatar", avatar);
        self
    }

    pub fn with_avatar_decoration(mut self, avatar_decoration: Option<String>) -> Self {
        self.updates.insert("avatarDecoration", avatar_decoration);
        self
    }

    pub fn with_bots_limit(mut self, bots_limit: i32) -> Self {
        self.updates.insert("botsLimit", bots_limit);
        self
    }

    pub fn with_mail(mut self, mail: String) -> Self {
        self.updates.insert("mail", mail);
        self
    }

    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.updates.insert("suspended", suspended);
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.updates.insert("username", username);
        self
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct UsersRepository {
    collection: Collection<User>,
}

impl UsersRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .iter()
            .any(|name| name == USERS_COLLECTION)
        {
            db.create_collection(USERS_COLLECTION).await?;
        }

        Ok(Self {
            collection: db.collection(USERS_COLLECTION),
        })
    }

    pub async fn find_all(&self) -> Result<Vec<User>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn count_users(&self) -> Result<u64> {
        self.collection.count_documents(doc! {}).await
    }

    pub async fn find_by_id(&self, user_id: &str) -> Result<Option<User>> {
        self.collection.find_one(doc! { "userId": user_id }).await
    }

    pub async fn find_many_by_ids(
        &self,
        user_ids: &HashSet<String>,
    ) -> Result<HashMap<String, User>> {
        let mut cursor = self
            .collection
            .find(doc! { "userId": { "$in": user_ids.iter().cloned().collect::<Vec<_>>() } })
            .await?;

        let mut map = HashMap::with_capacity(user_ids.len());

        while let Some(user) = cursor.try_next().await? {
            map.insert(user.user_id.clone(), user);
        }
        Ok(map)
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
