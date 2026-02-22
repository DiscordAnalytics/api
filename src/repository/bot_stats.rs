use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::BotStats, utils::constants::BOT_STATS_COLLECTION};

#[derive(Clone, Default)]
pub struct BotStatsUpdate {
    updates: Document,
}

impl BotStatsUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct BotStatsRepository {
    collection: Collection<BotStats>,
}

impl BotStatsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(BOT_STATS_COLLECTION),
        }
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<BotStats>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_date(&self, bot_id: &str, date: &DateTime) -> Result<Option<BotStats>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn find_by_date_range(
        &self,
        bot_id: &str,
        start_date: &DateTime,
        end_date: &DateTime,
    ) -> Result<Vec<BotStats>> {
        let cursor = self
            .collection
            .find(doc! { "botId": bot_id, "date": { "$gte": start_date, "$lte": end_date } })
            .await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, bot_stats: &BotStats) -> Result<InsertOneResult> {
        self.collection.insert_one(bot_stats).await
    }

    pub async fn update(
        &self,
        bot_id: &str,
        date: &DateTime,
        updated_bot_stats: BotStatsUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_bot_stats.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(
                doc! { "botId": bot_id, "date": date },
                doc! { "$set": updates },
            )
            .await
    }

    pub async fn delete_by_date(&self, bot_id: &str, date: &DateTime) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
