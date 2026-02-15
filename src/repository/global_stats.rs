use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, UpdateResult},
};

use crate::{domain::models::GlobalStats, utils::constants::GLOBAL_STATS_COLLECTION};

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

    pub async fn get_all(&self) -> Result<Vec<GlobalStats>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn get_for_date(&self, date: &DateTime) -> Result<Option<GlobalStats>> {
        self.collection.find_one(doc! { "date": date }).await
    }

    pub async fn update(&self, global_stats: &GlobalStats) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "date": &global_stats.date },
                doc! { "$set": serialize_to_document(global_stats)? },
            )
            .await
    }

    pub async fn delete(&self, date: &DateTime) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "date": date }).await
    }
}
