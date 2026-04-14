use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::TeamInvitation, utils::constants::TEAM_INVITATIONS_COLLECTION};

use super::common::{CollectionSpec, ensure_collection};

#[derive(Clone)]
pub struct TeamInvitationsRepository {
    collection: Collection<TeamInvitation>,
}

impl TeamInvitationsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        Ok(Self {
            collection: ensure_collection(
                db,
                TEAM_INVITATIONS_COLLECTION,
                CollectionSpec::Standard,
            )
            .await?,
        })
    }

    pub async fn find_all(&self) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, team_invitation_id: &str) -> Result<Option<TeamInvitation>> {
        self.collection
            .find_one(doc! { "invitationId": team_invitation_id })
            .await
    }

    pub async fn find_by_bot(&self, bot_id: &str) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_user(&self, user_id: &str) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! { "userId": user_id }).await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, team_invitation: &TeamInvitation) -> Result<InsertOneResult> {
        self.collection.insert_one(team_invitation).await
    }

    pub async fn accept_invitation(&self, invitation_id: &str) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "invitationId": invitation_id },
                doc! { "$set": { "accepted": true } },
            )
            .await
    }

    pub async fn delete_by_id(&self, team_invitation_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "invitationId": team_invitation_id })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }

    pub async fn delete_by_user_id(&self, user_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_many(doc! { "userId": user_id })
            .await
    }

    pub async fn delete_by_bot_and_user(
        &self,
        bot_id: &str,
        user_id: &str,
    ) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "botId": bot_id, "userId": user_id })
            .await
    }

    pub async fn delete_expired_invitations(&self) -> Result<u64> {
        let result = self
            .collection
            .delete_many(doc! { "expiration": { "$lte": DateTime::now() } })
            .await?;
        Ok(result.deleted_count)
    }
}
