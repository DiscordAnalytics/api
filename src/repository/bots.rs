use futures::stream::TryStreamExt as _;
use mongodb::{Collection, Database, bson::doc, error::Result};

use crate::utils::{constants::BOTS_COLLECTION, model::Bot};

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

    pub async fn find_by_id(&self, bot_id: &str) -> Result<Option<Bot>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Bot>> {
        let mut cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;
        let mut bots = Vec::new();
        while let Some(bot) = cursor.try_next().await? {
            bots.push(bot);
        }
        Ok(bots)
    }

    pub async fn is_suspended(&self, bot_id: &str) -> Result<bool> {
        let bot = self.find_by_id(bot_id).await?;
        Ok(bot.map_or(false, |b| b.suspended))
    }

    pub async fn update_framework(&self, bot_id: &str, framework: Option<String>) -> Result<()> {
        self.collection
            .update_one(
                doc! { "botId": bot_id },
                doc! { "$set": { "framework": framework } },
            )
            .await?;
        Ok(())
    }

    pub async fn delete(&self, bot_id: &str) -> Result<()> {
        self.collection.delete_one(doc! { "botId": bot_id }).await?;
        Ok(())
    }
}
