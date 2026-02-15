use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, UpdateResult},
};

use crate::{domain::models::StatsReport, utils::constants::STATS_REPORTS_COLLECTION};

#[derive(Clone)]
pub struct StatsReportsRepository {
    collection: Collection<StatsReport>,
}

impl StatsReportsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(STATS_REPORTS_COLLECTION),
        }
    }

    pub async fn get_all(&self) -> Result<Vec<StatsReport>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn get_for_bot(&self, bot_id: &str) -> Result<Vec<StatsReport>> {
        let cursor = self.collection.find(doc! { "bot_id": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn get_for_user(&self, user_id: &str) -> Result<Vec<StatsReport>> {
        let cursor = self.collection.find(doc! { "user_id": user_id }).await?;
        cursor.try_collect().await
    }

    pub async fn update(&self, stats_report: &StatsReport) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "_id": &stats_report.id },
                doc! { "$set": serialize_to_document(stats_report)? },
            )
            .await
    }

    pub async fn delete(&self, stats_report_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "_id": stats_report_id })
            .await
    }
}
