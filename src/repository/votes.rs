use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument, TimeseriesGranularity},
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::Vote, utils::constants::VOTES_COLLECTION};

use super::common::{CollectionSpec, ensure_collection};

#[derive(Clone)]
pub struct VotesRepository {
    collection: Collection<Vote>,
}

impl VotesRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(
                db,
                VOTES_COLLECTION,
                CollectionSpec::TimeSeries {
                    time_field: "date",
                    meta_field: Some("botId".to_owned()),
                    granularity: Some(TimeseriesGranularity::Hours),
                },
            )
            .await?,
        })
    }

    pub async fn find_by_date(&self, bot_id: &str, date: &DateTime) -> Result<Option<Vote>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn find_from_date_range(
        &self,
        bot_id: &str,
        from: &DateTime,
        to: &DateTime,
    ) -> Result<Option<Vote>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": { "$gte": from, "$lte": to } })
            .await
    }

    pub async fn count_votes_since(&self, bot_id: &str, since: &DateTime) -> Result<i64> {
        let mut cursor = self
            .collection
            .find(doc! { "botId": bot_id, "date": { "$gte": since } })
            .await?;
        let mut total = 0i64;
        while let Some(vote) = cursor.try_next().await? {
            total += vote.votes.values().map(|&count| count as i64).sum::<i64>();
        }
        Ok(total)
    }

    pub async fn insert(&self, vote: &Vote) -> Result<InsertOneResult> {
        self.collection.insert_one(vote).await
    }

    pub async fn increment_count(
        &self,
        bot_id: &str,
        date: &DateTime,
        provider: &str,
        increment_by: i32,
    ) -> Result<Option<Vote>> {
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                doc! { "botId": bot_id, "date": date },
                doc! { "$inc": { provider: increment_by } },
            )
            .with_options(options)
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
