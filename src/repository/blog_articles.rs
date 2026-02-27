use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{Document, doc},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::BlogArticle, utils::constants::BLOG_ARTICLES_COLLECTION};

#[derive(Clone, Default)]
pub struct BlogArticleUpdate {
    updates: Document,
}

impl BlogArticleUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Document {
        self.updates
    }
}

#[derive(Clone)]
pub struct BlogArticlesRepository {
    collection: Collection<BlogArticle>,
}

impl BlogArticlesRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(BLOG_ARTICLES_COLLECTION),
        }
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<BlogArticle>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_by_id(&self, article_id: &str) -> Result<Option<BlogArticle>> {
        self.collection
            .find_one(doc! { "articleId": article_id })
            .await
    }

    pub async fn insert(&self, article: &BlogArticle) -> Result<InsertOneResult> {
        self.collection.insert_one(article).await
    }

    pub async fn update(
        &self,
        article_id: &str,
        updated_article: BlogArticleUpdate,
    ) -> Result<UpdateResult> {
        let updates = updated_article.build();

        if updates.is_empty() {
            return Ok(UpdateResult::default());
        }

        self.collection
            .update_one(doc! { "articleId": article_id }, doc! { "$set": updates })
            .await
    }

    pub async fn delete(&self, article_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "articleId": article_id })
            .await
    }
}
