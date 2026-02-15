use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::TeamInvitation, utils::constants::TEAM_INVITATIONS_COLLECTION};

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

    pub async fn find_all(&self) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, team_invitation_id: &str) -> Result<Option<TeamInvitation>> {
        self.collection
            .find_one(doc! { "invitation_id": team_invitation_id })
            .await
    }

    pub async fn find_by_bot(&self, bot_id: &str) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! { "bot_id": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_user(&self, user_id: &str) -> Result<Vec<TeamInvitation>> {
        let cursor = self.collection.find(doc! { "user_id": user_id }).await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, team_invitation: &TeamInvitation) -> Result<InsertOneResult> {
        self.collection.insert_one(team_invitation).await
    }

    pub async fn update(&self, team_invitation: &TeamInvitation) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "invitation_id": &team_invitation.invitation_id },
                doc! { "$set": serialize_to_document(team_invitation)? },
            )
            .await
    }

    pub async fn delete(&self, team_invitation_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "invitation_id": team_invitation_id })
            .await
    }
}
