use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult},
};

use crate::{domain::models::Achievement, utils::constants::ACHIEVEMENTS_COLLECTION};

#[derive(Clone, Default)]
pub struct AchievementUpdate {
    updates: Document,
}

impl AchievementUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct AchievementsRepository {
    collection: Collection<Achievement>,
}

impl AchievementsRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .contains(&ACHIEVEMENTS_COLLECTION.to_string())
        {
            db.create_collection(ACHIEVEMENTS_COLLECTION).await?;
        }

        Ok(Self {
            collection: db.collection(ACHIEVEMENTS_COLLECTION),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<Achievement>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_all_shared(&self) -> Result<Vec<Achievement>> {
        let cursor = self
            .collection
            .find(doc! { "shared": true, "from": null })
            .await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, achievement_id: &str) -> Result<Option<Achievement>> {
        self.collection
            .find_one(doc! { "_id": achievement_id })
            .await
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<Achievement>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, achievement: &Achievement) -> Result<InsertOneResult> {
        self.collection.insert_one(achievement).await
    }

    pub async fn update(
        &self,
        achievement_id: &str,
        updated_achievement: AchievementUpdate,
    ) -> Result<Option<Achievement>> {
        let updates = updated_achievement.build();

        if updates.is_empty() {
            return Ok(None);
        }

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(doc! { "_id": achievement_id }, doc! { "$set": updates })
            .with_options(options)
            .await
    }

    pub async fn delete_by_id(&self, achievement_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "_id": achievement_id })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
