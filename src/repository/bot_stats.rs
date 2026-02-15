use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::BotStats, utils::constants::BOT_STATS_COLLECTION};

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

    pub async fn update(&self, bot_stats: &BotStats) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "botId": &bot_stats.bot_id, "date": &bot_stats.date },
                doc! { "$set": serialize_to_document(bot_stats)? },
            )
            .await
    }

    pub async fn delete(&self, bot_id: &str, date: &DateTime) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "date": date })
            .await
    }
}
