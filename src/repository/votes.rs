use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument, TimeseriesGranularity, TimeseriesOptions},
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::Vote, utils::constants::VOTES_COLLECTION};

#[derive(Clone, Default)]
pub struct VoteUpdate {
    updates: Document,
}

impl VoteUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct VotesRepository {
    collection: Collection<Vote>,
}

impl VotesRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .contains(&VOTES_COLLECTION.to_string())
        {
            let ts_opts = TimeseriesOptions::builder()
                .time_field("date")
                .meta_field(Some("botId".to_owned()))
                .granularity(Some(TimeseriesGranularity::Hours))
                .build();
            db.create_collection(VOTES_COLLECTION)
                .timeseries(ts_opts)
                .await?;
        }

        Ok(Self {
            collection: db.collection(VOTES_COLLECTION),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<Vote>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_bot(&self, bot_id: &str) -> Result<Vec<Vote>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_date_and_provider(
        &self,
        bot_id: &str,
        date: &DateTime,
        provider: &str,
    ) -> Result<Option<Vote>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date, "provider": provider })
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
            total += vote.count as i64;
        }
        Ok(total)
    }

    pub async fn insert(&self, vote: &Vote) -> Result<InsertOneResult> {
        self.collection.insert_one(vote).await
    }

    pub async fn update(
        &self,
        bot_id: &str,
        date: &DateTime,
        updated_vote: VoteUpdate,
    ) -> Result<Option<Vote>> {
        let updates = updated_vote.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                doc! { "botId": bot_id, "date": date },
                doc! { "$set": updates },
            )
            .with_options(options)
            .await
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
                doc! { "botId": bot_id, "date": date, "provider": provider },
                doc! { "$inc": { "count": increment_by } },
            )
            .with_options(options)
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
