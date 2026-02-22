use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::Bot, utils::constants::BOTS_COLLECTION};

#[derive(Clone, Default)]
pub struct BotUpdate {
    updates: Document,
}

impl BotUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_avatar(mut self, avatar: String) -> Self {
        self.updates.insert("avatar", avatar);
        self
    }

    pub fn with_framework(mut self, framework: String) -> Self {
        self.updates.insert("framework", framework);
        self
    }

    pub fn with_team(mut self, team: Vec<String>) -> Self {
        self.updates.insert("team", team);
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.updates.insert("username", username);
        self
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.updates.insert("version", version);
        self
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct BotsRepository {
    collection: Collection<Bot>,
}

impl BotsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(BOTS_COLLECTION),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, bot_id: &str) -> Result<Option<Bot>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;
        cursor.try_collect().await
    }

    pub async fn count_by_user_id(&self, user_id: &str) -> Result<u64> {
        self.collection
            .count_documents(doc! { "ownerId": user_id })
            .await
    }

    pub async fn insert(&self, bot: &Bot) -> Result<InsertOneResult> {
        self.collection.insert_one(bot).await
    }

    pub async fn update(&self, bot_id: &str, updated_bot: BotUpdate) -> Result<UpdateResult> {
        let updates = updated_bot.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(doc! { "botId": bot_id }, doc! { "$set": updates })
            .await
    }

    pub async fn delete(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "botId": bot_id }).await
    }

    pub async fn delete_by_user_id(&self, user_id: &str) -> Result<()> {
        self.collection
            .delete_many(doc! { "ownerId": user_id })
            .await?;
        Ok(())
    }
}
