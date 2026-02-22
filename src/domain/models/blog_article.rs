use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogArticle {
    pub author_id: String,
    pub article_id: String,
    pub content: String,
    pub cover: Option<String>,
    pub created_at: DateTime,
    pub description: String,
    pub is_draft: bool,
    pub tags: Vec<String>,
    pub title: String,
    pub updated_at: Option<DateTime>,
}
