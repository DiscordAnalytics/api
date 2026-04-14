use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::doc,
    error::Result,
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::StatsReport, utils::constants::STATS_REPORTS_COLLECTION};

use super::common::{CollectionSpec, ensure_collection};

#[derive(Clone)]
pub struct StatsReportsRepository {
    collection: Collection<StatsReport>,
}

impl StatsReportsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, STATS_REPORTS_COLLECTION, CollectionSpec::Standard)
                .await?,
        })
    }

    pub async fn find_by_bot(&self, bot_id: &str) -> Result<Vec<StatsReport>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_bot_and_user(
        &self,
        bot_id: &str,
        user_id: &str,
        frequency: &str,
    ) -> Result<Option<StatsReport>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "userId": user_id, "frequency": frequency })
            .await
    }

    pub async fn find_by_frequency(&self, frequency: &str) -> Result<Vec<StatsReport>> {
        let cursor = self
            .collection
            .find(doc! { "frequency": frequency })
            .await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, stats_report: &StatsReport) -> Result<InsertOneResult> {
        self.collection.insert_one(stats_report).await
    }

    pub async fn delete_by_bot(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }

    pub async fn delete_by_user(&self, user_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_many(doc! { "userId": user_id })
            .await
    }

    pub async fn delete_by_bot_and_user(
        &self,
        bot_id: &str,
        user_id: &str,
        frequency: &str,
    ) -> Result<DeleteResult> {
        self.collection
            .delete_many(doc! { "botId": bot_id, "userId": user_id, "frequency": frequency })
            .await
    }
}
