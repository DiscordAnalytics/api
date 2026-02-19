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

impl BlogArticle {
    pub fn with_author_id(mut self, author_id: String) -> Self {
        self.author_id = author_id;
        self
    }

    pub fn with_article_id(mut self, article_id: String) -> Self {
        self.article_id = article_id;
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn with_cover(mut self, cover: Option<String>) -> Self {
        self.cover = cover;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime) -> Self {
        self.created_at = created_at;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_is_draft(mut self, is_draft: bool) -> Self {
        self.is_draft = is_draft;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_updated_at(mut self, updated_at: Option<DateTime>) -> Self {
        self.updated_at = updated_at;
        self
    }
}
