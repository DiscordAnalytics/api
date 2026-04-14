use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc},
    error::Result,
    results::{InsertOneResult, UpdateResult},
};

use crate::{domain::models::Session, utils::constants::SESSIONS_COLLECTION};

use super::common::{CollectionSpec, ensure_collection};

#[derive(Clone)]
pub struct SessionsRepository {
    collection: Collection<Session>,
}

impl SessionsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(db, SESSIONS_COLLECTION, CollectionSpec::Standard)
                .await?,
        })
    }

    pub async fn insert(&self, session: &Session) -> Result<InsertOneResult> {
        self.collection.insert_one(session).await
    }

    pub async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        self.collection
            .find_one(doc! { "sessionId": session_id })
            .await
    }

    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>> {
        let cursor = self
            .collection
            .find(doc! {
              "active": true,
              "expiresAt": { "$gt": DateTime::now() },
              "userId": user_id,
            })
            .await?;
        cursor.try_collect().await
    }

    pub async fn update_last_used(&self, session_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "sessionId": session_id },
                doc! { "$set": { "lastUsedAt": DateTime::now() } },
            )
            .await
    }

    pub async fn revoke(&self, session_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "sessionId": session_id },
                doc! { "$set": { "active": false } },
            )
            .await
    }

    pub async fn revoke_many_for_user(
        &self,
        user_id: &str,
        ignored_session_id: &str,
    ) -> Result<UpdateResult> {
        self.collection
            .update_many(
                doc! { "userId": user_id, "sessionId": { "$ne": ignored_session_id } },
                doc! { "$set": { "active": false } },
            )
            .await
    }

    pub async fn revoke_all_for_user(&self, user_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_many(
                doc! { "userId": user_id },
                doc! { "$set": { "active": false } },
            )
            .await
    }

    pub async fn delete_expired(&self) -> Result<u64> {
        let result = self
            .collection
            .delete_many(doc! {
                "expiresAt": { "$lt": DateTime::now() },
            })
            .await?;
        Ok(result.deleted_count)
    }
}
