mod achievements;
mod blog_articles;
mod bot_stats;
mod bots;
mod common;
mod connection;
mod custom_events;
#[cfg(feature = "reports")]
mod r2;
mod sessions;
#[cfg(feature = "reports")]
mod stats_reports;
mod status_check;
mod status_current;
mod status_incidents;
mod team_invitations;
mod users;
mod votes;

use anyhow::{Context, Result, bail};
use mongodb::{Client, Database, bson::doc};

pub use achievements::AchievementUpdate;
pub use blog_articles::BlogArticleUpdate;
pub use bot_stats::BotStatsUpdate;
pub use bots::BotUpdate;
pub use custom_events::CustomEventUpdate;
pub use users::UserUpdate;

#[derive(Clone)]
pub struct Repositories {
    pub client: Client,

    pub achievements: achievements::AchievementsRepository,
    pub blog_articles: blog_articles::BlogArticlesRepository,
    pub bots: bots::BotsRepository,
    pub bot_stats: bot_stats::BotStatsRepository,
    pub custom_events: custom_events::CustomEventsRepository,
    database: connection::DbConnection,
    pub sessions: sessions::SessionsRepository,
    #[cfg(feature = "reports")]
    pub r2: r2::R2Repository,
    #[cfg(feature = "reports")]
    pub stats_reports: stats_reports::StatsReportsRepository,
    pub status_check: status_check::StatusCheckRepository,
    pub status_current: status_current::StatusCurrentRepository,
    pub status_incidents: status_incidents::StatusIncidentsRepository,
    pub team_invitations: team_invitations::TeamInvitationsRepository,
    pub users: users::UsersRepository,
    pub votes: votes::VotesRepository,
}

impl Repositories {
    pub async fn init() -> Result<Self> {
        let connection = connection::DbConnection::init().await?;
        let db = connection.database();
        let client = db.client().clone();

        Self::ensure_transaction_support(db).await?;

        Ok(Self {
            client,

            achievements: achievements::AchievementsRepository::new(db).await?,
            blog_articles: blog_articles::BlogArticlesRepository::new(db).await?,
            bots: bots::BotsRepository::new(db).await?,
            bot_stats: bot_stats::BotStatsRepository::new(db).await?,
            custom_events: custom_events::CustomEventsRepository::new(db).await?,
            database: connection.clone(),
            sessions: sessions::SessionsRepository::new(db).await?,
            #[cfg(feature = "reports")]
            r2: r2::R2Repository::new()?,
            #[cfg(feature = "reports")]
            stats_reports: stats_reports::StatsReportsRepository::new(db).await?,
            status_check: status_check::StatusCheckRepository::new(db).await?,
            status_current: status_current::StatusCurrentRepository::new(db).await?,
            status_incidents: status_incidents::StatusIncidentsRepository::new(db).await?,
            team_invitations: team_invitations::TeamInvitationsRepository::new(db).await?,
            users: users::UsersRepository::new(db).await?,
            votes: votes::VotesRepository::new(db).await?,
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.database
            .ping()
            .await
            .map_err(|e| anyhow::anyhow!("Database ping failed: {}", e))?;
        #[cfg(feature = "reports")]
        self.r2
            .ping()
            .await
            .map_err(|e| anyhow::anyhow!("R2 ping failed: {}", e))?;

        Ok(())
    }

    async fn ensure_transaction_support(db: &Database) -> Result<()> {
        let hello = db
            .run_command(doc! { "hello": 1})
            .await
            .context("Failed to check MongoDB deployment capabilities")?;

        let support_transactions = hello.contains_key("setName")
            || hello.get_str("msg").ok() == Some("isdbgrid")
            || hello.contains_key("serviceId");

        if !support_transactions {
            bail!(
                "Mongodb deployment does not support transactions. A replica set or sharded cluster is required."
            );
        }

        Ok(())
    }
}
