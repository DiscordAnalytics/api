use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::TeamInvitation, utils::constants::TEAM_INVITATIONS_COLLECTION};

#[derive(Clone, Default)]
pub struct TeamInvitationUpdate {
    updates: Document,
}

impl TeamInvitationUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct TeamInvitationsRepository {
    collection: Collection<TeamInvitation>,
}

impl TeamInvitationsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(TEAM_INVITATIONS_COLLECTION),
        }
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
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

    pub async fn update(
        &self,
        invitation_id: &str,
        updated_team_invitation: TeamInvitationUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_team_invitation.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(
                doc! { "invitationId": invitation_id },
                doc! { "$set": updates },
            )
            .await
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
}
