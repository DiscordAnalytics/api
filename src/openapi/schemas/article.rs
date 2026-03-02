use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::BlogArticle;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArticleResponse {
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

impl TryFrom<BlogArticle> for ArticleResponse {
    type Error = anyhow::Error;

    fn try_from(value: BlogArticle) -> Result<Self, Self::Error> {
        Ok(Self {
            author_id: value.author_id,
            article_id: value.article_id,
            content: Some(value.content),
            cover: value.cover,
            created_at: value.created_at.try_to_rfc3339_string()?,
            description: value.description,
            is_draft: value.is_draft,
            tags: value.tags,
            title: value.title,
            updated_at: value
                .updated_at
                .map(|dt| dt.try_to_rfc3339_string())
                .transpose()?,
        })
    }
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
