use futures::stream::TryStreamExt as _;
use mongodb::{
    Collection, Database,
    bson::{doc, serialize_to_document},
    error::Result,
    results::{DeleteResult, InsertOneResult, UpdateResult},
};

use crate::{domain::models::BlogArticle, utils::constants::BLOG_ARTICLES_COLLECTION};

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
        updated_article: &BlogArticle,
    ) -> Result<UpdateResult> {
        self.collection
            .update_one(
                doc! { "articleId": article_id },
                doc! { "$set": serialize_to_document(updated_article)? },
            )
            .await
    }

    pub async fn delete(&self, article_id: &str) -> Result<DeleteResult> {
        self.collection
            .delete_one(doc! { "articleId": article_id })
            .await
    }
}
