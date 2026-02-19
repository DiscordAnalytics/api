use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::Achievement, utils::constants::ACHIEVEMENTS_COLLECTION};

#[derive(Clone)]
pub struct AchievementsRepository {
    collection: Collection<Achievement>,
}

impl AchievementsRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(ACHIEVEMENTS_COLLECTION),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<Achievement>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, achievement_id: &str) -> Result<Option<Achievement>> {
        self.collection
            .find_one(doc! { "_id": achievement_id })
            .await
    }

    pub async fn insert(&self, achievement: &Achievement) -> Result<InsertOneResult> {
        self.collection.insert_one(achievement).await
    }

    pub async fn update(&self, achievement: &Achievement) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "_id": &achievement.id },
                doc! { "$set": serialize_to_document(achievement)? },
            )
            .await
    }

    pub async fn delete(&self, achievement_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "_id": achievement_id })
            .await
    }
}
