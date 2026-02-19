use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::Bot, utils::constants::BOTS_COLLECTION};

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

    pub async fn insert(&self, bot: &Bot) -> Result<InsertOneResult> {
        self.collection.insert_one(bot).await
    }

    pub async fn update(&self, updated_bot: &Bot) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "botId": &updated_bot.bot_id },
                doc! { "$set": serialize_to_document(updated_bot)? },
            )
            .await
    }

    pub async fn delete(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "botId": bot_id }).await
    }
}
