use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::CustomEvent, utils::constants::CUSTOM_EVENTS_COLLECTION};

use super::common::{UpdateBuilder, ensure_collection};

#[derive(Clone, Default)]
pub struct CustomEventUpdate {
    builder: UpdateBuilder,
}

impl CustomEventUpdate {
    pub fn with_graph_name(mut self, graph_name: &str) -> Self {
        self.builder = self.builder.set(doc! { "graphName": graph_name });
        self
    }

    pub fn build(self) -> Document {
        self.builder.build()
    }
}

#[derive(Clone)]
pub struct CustomEventsRepository {
    collection: Collection<CustomEvent>,
}

impl CustomEventsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, CUSTOM_EVENTS_COLLECTION).await?,
        })
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<CustomEvent>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_bot_id_and_event_key(
        &self,
        bot_id: &str,
        event_key: &str,
    ) -> Result<Option<CustomEvent>> {
        self.collection
            .find_one(doc! { "botId": bot_id, "eventKey": event_key })
            .await
    }

    pub async fn insert(&self, custom_event: &CustomEvent) -> Result<InsertOneResult> {
        self.collection.insert_one(custom_event).await
    }

    pub async fn update(
        &self,
        bot_id: &str,
        event_key: &str,
        updated_custom_event: CustomEventUpdate,
    ) -> Result<Option<CustomEvent>> {
        let updates = updated_custom_event.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                doc! { "botId": bot_id, "eventKey": event_key },
                doc! { "$set": updates },
            )
            .with_options(options)
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
