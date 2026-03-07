use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::BlogArticle;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponse {
    pub author: Option<ArticleAuthor>,
    pub author_id: String,
    pub article_id: String,
    pub content: Option<String>,
    pub cover: Option<String>,
    pub created_at: String,
    pub description: String,
    pub is_draft: bool,
    pub tags: Vec<String>,
    pub title: String,
    pub updated_at: Option<String>,
}

impl ArticleResponse {
    pub fn from_article(
        article: BlogArticle,
        author: Option<ArticleAuthor>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            author,
            author_id: article.author_id,
            article_id: article.article_id,
            content: Some(article.content),
            cover: article.cover,
            created_at: article.created_at.try_to_rfc3339_string()?,
            description: article.description,
            is_draft: article.is_draft,
            tags: article.tags,
            title: article.title,
            updated_at: article
                .updated_at
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct ArticleAuthor {
    pub avatar: Option<String>,
    pub username: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct ArticleRequest {
    pub content: String,
    pub cover: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub title: String,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct ArticleDeleteResponse {
    pub success: bool,
}
