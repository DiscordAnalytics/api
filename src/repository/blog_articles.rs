use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
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

    pub fn with_content(mut self, content: &str) -> Self {
        self.updates.insert("content", content);
        self
    }

    pub fn with_cover(mut self, cover: &str) -> Self {
        self.updates.insert("cover", cover);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.updates.insert("description", description);
        self
    }

    pub fn with_is_draft(mut self, is_draft: bool) -> Self {
        self.updates.insert("isDraft", is_draft);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.updates.insert("tags", tags);
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.updates.insert("title", title);
        self
    }

    pub fn with_updated_at_to_now(mut self) -> Self {
        self.updates.insert("updatedAt", DateTime::now());
        self
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
    pub async fn new(db: &Database) -> Result<Self> {
        if !db
            .list_collection_names()
            .await?
            .contains(&BLOG_ARTICLES_COLLECTION.to_string())
        {
            db.create_collection(BLOG_ARTICLES_COLLECTION).await?;
        }

        Ok(Self {
            collection: db.collection(BLOG_ARTICLES_COLLECTION),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.collection.find_one(doc! {}).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<BlogArticle>> {
        let cursor = self.collection.find(doc! {}).await?;
        cursor.try_collect().await
    }

    pub async fn find_all_published(&self) -> Result<Vec<BlogArticle>> {
        let cursor = self.collection.find(doc! { "isDraft": false }).await?;
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
            .update_one(
                doc! {
                  "articleId": article_id
                },
                doc! { "$set": updates },
            )
            .await
    }

    pub async fn delete(&self, article_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "articleId": article_id })
            .await
    }
}
