use futures::stream::TryStreamExt as _;
use mongodb::{Collection, Database, bson::doc, error::Result, results::DeleteResult};

use crate::utils::{constants::BOTS_COLLECTION, model::Bot};

#[derive(Clone)]
pub struct BotsRepository {
    collection: Collection<Bot>,
}

impl BotsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(BOTS_COLLECTION),
        }
    }

    pub async fn find_by_id(&self, bot_id: &str) -> Result<Option<Bot>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Bot>> {
        let cursor = self.collection.find(doc! { "ownerId": owner_id }).await?;
        cursor.try_collect().await
    }

    pub async fn delete(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_one(doc! { "botId": bot_id }).await
    }
}
