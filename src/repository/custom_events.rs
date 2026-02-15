use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, UpdateResult},
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

    pub async fn get_all_for_bot(&self, bot_id: &str) -> Result<Vec<CustomEvent>> {
        let cursor = self.collection.find(doc! { "bot_id": bot_id }).await?;
        cursor.try_collect().await
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
