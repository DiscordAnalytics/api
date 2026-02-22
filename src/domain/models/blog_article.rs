use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BlogArticle {
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "articleId")]
    pub article_id: String,
    pub content: String,
    pub cover: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    pub description: String,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub tags: Vec<String>,
    pub title: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime>,
}
