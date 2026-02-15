mod achievements;
mod blog_articles;
mod bots;
mod connection;

use crate::repository::{
    achievements::AchievementsRepository, blog_articles::BlogArticlesRepository,
    bots::BotsRepository, connection::DbConnection,
};

#[derive(Clone)]
pub struct Repositories {
    pub achievements: AchievementsRepository,
    pub blog_articles: BlogArticlesRepository,
    pub bots: BotsRepository,
}

impl Repositories {
    pub async fn init() -> Result<Self, mongodb::error::Error> {
        let connection = DbConnection::init().await?;
        let db = connection.database();

        Ok(Self {
            achievements: AchievementsRepository::new(db),
            blog_articles: BlogArticlesRepository::new(db),
            bots: BotsRepository::new(db),
        })
    }
}
