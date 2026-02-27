use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
    error::Result,
    options::FindOptions,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::GlobalStats, utils::constants::GLOBAL_STATS_COLLECTION};

#[derive(Clone, Default)]
pub struct GlobalStatsUpdate {
    updates: Document,
}

impl GlobalStatsUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bot_count(mut self, bot_count: i32) -> Self {
        self.updates.insert("botCount", bot_count);
        self
    }

    pub fn with_registered_bots(mut self, registered_bots: i32) -> Self {
        self.updates.insert("registeredBots", registered_bots);
        self
    }

    pub fn with_user_count(mut self, user_count: i32) -> Self {
        self.updates.insert("userCount", user_count);
        self
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct GlobalStatsRepository {
    collection: Collection<GlobalStats>,
}

impl GlobalStatsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(GLOBAL_STATS_COLLECTION),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<GlobalStats>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_one(&self, date: &DateTime) -> Result<Option<GlobalStats>> {
        self.collection.find_one(doc! { "date": date }).await
    }

    pub async fn find_from_date_range(
        &self,
        start_date: &DateTime,
        end_date: &DateTime,
    ) -> Result<Vec<GlobalStats>> {
        let options = FindOptions::builder().sort(doc! { "date": 1 }).build();

        let cursor = self
            .collection
            .find(doc! {
                    "date": {
                        "$gte": start_date,
                        "$lte": end_date,
                    }
            })
            .with_options(options)
            .await?;

        cursor.try_collect().await
    }

    pub async fn insert(&self, global_stats: &GlobalStats) -> Result<InsertOneResult> {
        self.collection.insert_one(global_stats).await
    }

    pub async fn update(
        &self,
        date: &DateTime,
        updated_global_stats: GlobalStatsUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_global_stats.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(doc! { "date": date }, doc! { "$set": updates })
            .await
    }

    pub async fn delete(&self, date: &DateTime) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "date": date }).await
    }
}
