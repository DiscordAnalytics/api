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

impl BlogArticle {
    pub fn new(
        author_id: &str,
        content: &str,
        description: &str,
        tags: Vec<String>,
        title: &str,
    ) -> Self {
        Self {
            author_id: author_id.to_string(),
            article_id: Self::generate_article_id(title),
            content: content.to_string(),
            cover: None,
            created_at: DateTime::now(),
            description: description.to_string(),
            is_draft: true,
            tags,
            title: title.to_string(),
            updated_at: None,
        }
    }

    pub fn with_cover(mut self, cover: &str) -> Self {
        self.cover = Some(cover.to_string());
        self
    }

    pub fn generate_article_id(title: &str) -> String {
        let timestamp = DateTime::now().timestamp_millis();
        let sanitized_title = title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .map(|c| if c.is_whitespace() { '-' } else { c })
            .collect::<String>();

        let article_id = format!("{}-{}", timestamp, sanitized_title);
        article_id.chars().take(100).collect()
    }
}
