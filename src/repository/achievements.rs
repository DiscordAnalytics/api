use std::str::FromStr;

use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Bson, DateTime, Document, doc, oid::ObjectId},
    error::Result,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    results::{DeleteResult, InsertOneResult, UpdateResult},
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

    pub fn with_description(mut self, description: String) -> Self {
        self.merge_set(doc! { "description": description });
        self
    }

    pub fn with_from(mut self, from: Option<String>) -> Self {
        self.merge_set(doc! { "from": from });
        self
    }

    pub fn with_lang(mut self, lang: String) -> Self {
        self.merge_set(doc! { "lang": lang });
        self
    }

    pub fn with_shared(mut self, shared: bool) -> Self {
        self.merge_set(doc! { "shared": shared });
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.merge_set(doc! { "title": title });
        self
    }

    pub fn with_used_by(mut self, used_by: i64) -> Self {
        self.merge_set(doc! { "usedBy": used_by });
        self
    }

    fn merge_set(&mut self, doc: Document) {
        let set_doc = self
            .updates
            .entry("$set")
            .or_insert_with(|| Bson::Document(doc! {}));

        if let Bson::Document(existing) = set_doc {
            existing.extend(doc);
        }
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

    pub async fn count_used_by(&self, achievement_id: &str) -> Result<u64> {
        self.collection
            .count_documents(doc! { "from": achievement_id })
            .await
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
            .find_one(doc! { "_id": ObjectId::from_str(achievement_id)? })
            .await
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Vec<Achievement>> {
        let cursor = self.collection.find(doc! { "botId": bot_id }).await?;
        cursor.try_collect().await
    }

    pub async fn find_unachieved_by_bot(&self, bot_id: &str) -> Result<Vec<Achievement>> {
        let cursor = self
            .collection
            .find(doc! { "botId": bot_id, "achievedOn": null })
            .await?;
        cursor.try_collect().await
    }

    pub async fn insert(&self, achievement: &Achievement) -> Result<InsertOneResult> {
        self.collection.insert_one(achievement).await
    }

    pub async fn insert_many(&self, achievements: &[Achievement]) -> Result<()> {
        if achievements.is_empty() {
            return Ok(());
        }

        self.collection.insert_many(achievements).await?;
        Ok(())
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
            .find_one_and_update(doc! { "_id": ObjectId::from_str(achievement_id)? }, updates)
            .with_options(options)
            .await
    }

    pub async fn update_many(
        &self,
        from_achievement_id: &str,
        updated_achievement: AchievementUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_achievement.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_many(doc! { "from": from_achievement_id }, updates)
            .await
    }

    pub async fn update_progress(
        &self,
        bot_id: &str,
        achievement_id: &str,
        current: Option<i64>,
        achieved_on: Option<DateTime>,
    ) -> Result<Option<Achievement>> {
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                doc! { "_id": ObjectId::from_str(achievement_id)?, "botId": bot_id },
                doc! { "$set": { "current": current, "achievedOn": achieved_on } },
            )
            .with_options(options)
            .await
    }

    pub async fn delete_by_id(&self, achievement_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "_id": ObjectId::from_str(achievement_id)? })
            .await
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<DeleteResult> {
        self.collection.delete_many(doc! { "botId": bot_id }).await
    }
}
