use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::Vote, utils::constants::VOTES_COLLECTION};

#[derive(Clone)]
pub struct VotesRepository {
    collection: Collection<Vote>,
}

impl VotesRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(VOTES_COLLECTION),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<Vote>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_bot(&self, bot_id: &str) -> Result<Vec<Vote>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_bot_and_date(
        &self,
        bot_id: &str,
        date: &DateTime,
    ) -> Result<Option<Vote>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "date": date })
            .await
    }

    pub async fn insert(&self, vote: &Vote) -> Result<InsertOneResult> {
        self.collection.insert_one(vote).await
    }

    pub async fn update(&self, vote: &Vote) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "botId": &vote.bot_id, "date": &vote.date },
                doc! { "$set": serialize_to_document(vote)? },
            )
            .await
    }

    pub async fn delete(&self, bot_id: &str, date: &DateTime) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "date": date })
            .await
    }
}
