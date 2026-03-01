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

    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.updates.insert("suspended", suspended);
        self
    }

    pub fn with_team(mut self, team: Vec<String>) -> Self {
        self.updates.insert("team", team);
        self
    }

    pub fn with_team_member(mut self, user_id: &str) -> Self {
        self.updates.insert("$push", doc! { "team": user_id });
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.updates.insert("token", token);
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
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .contains(&BOTS_COLLECTION.to_string())
        {
            db.create_collection(BOTS_COLLECTION).await?;
        }

        Ok(Self {
            collection: db.collection(BOTS_COLLECTION),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn count_bots(&self) -> Result<u64> {
        self.collection.count_documents(doc! {}).await
    }

    pub async fn find_by_id(&self, bot_id: &str) -> Result<Option<Bot>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn find_by_owner_id(&self, owner_id: &str) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_team_member_id(&self, user_id: &str) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "team": user_id }).await?;
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

    pub async fn add_user_to_team(&self, bot_id: &str, user_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "botId": bot_id },
                doc! { "$addToSet": { "team": user_id } },
            )
            .await
    }

    pub async fn remove_user_from_team(&self, bot_id: &str, user_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "botId": bot_id },
                doc! { "$pull": { "team": user_id } },
            )
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
