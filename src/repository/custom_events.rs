use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::CustomEvent, utils::constants::CUSTOM_EVENTS_COLLECTION};

#[derive(Clone, Default)]
pub struct CustomEventUpdate {
    updates: Document,
}

impl CustomEventUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct CustomEventsRepository {
    collection: Collection<CustomEvent>,
}

impl CustomEventsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(CUSTOM_EVENTS_COLLECTION),
        }
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<CustomEvent>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_event_key(&self, event_key: &str) -> Result<Option<CustomEvent>> {
        self.collection
            .find_one(doc! { "eventKey": event_key })
            .await
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<CustomEvent>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, custom_event: &CustomEvent) -> Result<InsertOneResult> {
        self.collection.insert_one(custom_event).await
    }

    pub async fn update(
        &self,
        bot_id: &str,
        event_key: &str,
        updated_custom_event: CustomEventUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_custom_event.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(
                doc! { "botId": bot_id, "eventKey": event_key },
                doc! { "$set": updates },
            )
            .await
    }

    pub async fn delete_by_event_key(&self, bot_id: &str, event_key: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "eventKey": event_key })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
