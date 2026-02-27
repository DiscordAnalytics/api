use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
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

    pub fn with_avatar(mut self, avatar: String) -> Self {
        self.updates.insert("avatar", avatar);
        self
    }

    pub fn with_avatar_decoration(mut self, avatar_decoration: String) -> Self {
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
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(USERS_COLLECTION),
        }
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
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

    pub async fn insert(&self, user: &User) -> Result<InsertOneResult> {
        self.collection.insert_one(user).await
    }

    pub async fn update(&self, user_id: &str, updated_user: UserUpdate) -> Result<UpdateResult> {
        let updates = updated_user.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(doc! { "userId": user_id }, doc! { "$set": updates })
            .await
    }

    pub async fn delete_by_id(&self, user_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "userId": user_id }).await
    }
}
