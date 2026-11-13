use std::time::Duration;

use futures::StreamExt;
use mongodb::{
    ClientSession, Collection, Database, IndexModel,
    bson::{DateTime, doc},
    error::Result,
    options::IndexOptions,
};

use crate::{
    domain::models::{StatusCheck, StoredStatusCheck},
    openapi::schemas::{HistoryCursor, HistoryPage},
    repository::common::ensure_collection,
    utils::constants::STATUS_CHECKS_COLLECTION,
};

#[derive(Clone)]
pub struct StatusCheckRepository {
    collection: Collection<StatusCheck>,
}

impl StatusCheckRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        let collection = ensure_collection(db, STATUS_CHECKS_COLLECTION).await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "botId": 1, "timestamp": -1, "_id": -1 })
                    .build(),
            )
            .await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "timestamp": 1 })
                    .options(
                        IndexOptions::builder()
                            .expire_after(Duration::from_secs(7776000))
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(Self { collection })
    }

    pub async fn create_check(&self, check: StatusCheck) -> Result<()> {
        self.collection.insert_one(check).await?;
        Ok(())
    }

    pub async fn create_check_with_session(
        &self,
        check: StatusCheck,
        session: &mut ClientSession,
    ) -> Result<()> {
        self.collection
            .insert_one(check)
            .session(&mut *session)
            .await?;
        Ok(())
    }

    pub async fn find_history(
        &self,
        bot_id: &str,
        from: DateTime,
        to: DateTime,
        limit: u32,
        cursor: Option<HistoryCursor>,
    ) -> Result<HistoryPage> {
        let mut filter = doc! {
            "botId": bot_id,
            "timestamp": {
              "$gte": from,
              "$lt": to,
            },
        };

        if let Some(cursor) = cursor {
            filter.insert(
                "$or",
                vec![
                    doc! {
                      "timestamp": {
                        "$lt": cursor.timestamp,
                      }
                    },
                    doc! {
                      "timestamp": cursor.timestamp,
                      "_id": {
                        "$lt": cursor.id,
                      },
                    },
                ],
            );
        }

        let collection = self.collection.clone_with_type::<StoredStatusCheck>();

        let mut cursor = collection
            .find(filter)
            .sort(doc! {
              "timestamp": -1,
              "_id": -1,
            })
            .limit(i64::from(limit) + 1)
            .await?;

        let mut checks = Vec::with_capacity(limit as usize + 1);

        while let Some(result) = cursor.next().await {
            checks.push(result?);
        }

        let has_more = checks.len() > limit as usize;

        if has_more {
            checks.pop();
        }

        Ok(HistoryPage { checks, has_more })
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<()> {
        self.collection
            .delete_many(doc! { "botId": bot_id })
            .await?;
        Ok(())
    }
}
