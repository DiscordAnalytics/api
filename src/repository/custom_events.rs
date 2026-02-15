use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::CustomEvent, utils::constants::CUSTOM_EVENTS_COLLECTION};

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

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<CustomEvent>> {
        let cursor = self.collection.find(doc! { "bot_id": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, custom_event: &CustomEvent) -> Result<InsertOneResult> {
        self.collection.insert_one(custom_event).await
    }

    pub async fn update(&self, custom_event: &CustomEvent) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "bot_id": &custom_event.bot_id, "event_key": &custom_event.event_key },
                doc! { "$set": serialize_to_document(custom_event)? },
            )
            .await
    }

    pub async fn delete(&self, bot_id: &str, event_key: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "bot_id": bot_id, "event_key": event_key })
            .await
    }
}
