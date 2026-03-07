mod achievements;
mod blog_articles;
mod bot_stats;
mod bots;
mod connection;
mod custom_events;
mod global_stats;
#[cfg(feature = "reports")]
mod r2;
mod sessions;
#[cfg(feature = "reports")]
mod stats_reports;
mod team_invitations;
mod users;
mod votes;

pub use achievements::AchievementUpdate;
use anyhow::Result;
pub use blog_articles::BlogArticleUpdate;
pub use bot_stats::BotStatsUpdate;
pub use bots::BotUpdate;
pub use custom_events::CustomEventUpdate;
pub use global_stats::GlobalStatsUpdate;
#[cfg(feature = "reports")]
pub use stats_reports::StatsReportUpdate;
pub use team_invitations::TeamInvitationUpdate;
pub use users::UserUpdate;

#[derive(Clone)]
pub struct Repositories {
    pub achievements: achievements::AchievementsRepository,
    pub blog_articles: blog_articles::BlogArticlesRepository,
    pub bots: bots::BotsRepository,
    pub bot_stats: bot_stats::BotStatsRepository,
    pub custom_events: custom_events::CustomEventsRepository,
    pub global_stats: global_stats::GlobalStatsRepository,
    pub sessions: sessions::SessionsRepository,
    #[cfg(feature = "reports")]
    pub r2: r2::R2Repository,
    #[cfg(feature = "reports")]
    pub stats_reports: stats_reports::StatsReportsRepository,
    pub team_invitations: team_invitations::TeamInvitationsRepository,
    pub users: users::UsersRepository,
    pub votes: votes::VotesRepository,
}

impl Repositories {
    pub async fn init() -> Result<Self> {
        let connection = connection::DbConnection::init().await?;
        let db = connection.database();

        Ok(Self {
            achievements: achievements::AchievementsRepository::new(db).await?,
            blog_articles: blog_articles::BlogArticlesRepository::new(db).await?,
            bots: bots::BotsRepository::new(db).await?,
            bot_stats: bot_stats::BotStatsRepository::new(db).await?,
            custom_events: custom_events::CustomEventsRepository::new(db).await?,
            global_stats: global_stats::GlobalStatsRepository::new(db).await?,
            sessions: sessions::SessionsRepository::new(db).await?,
            #[cfg(feature = "reports")]
            r2: r2::R2Repository::new()?,
            #[cfg(feature = "reports")]
            stats_reports: stats_reports::StatsReportsRepository::new(db).await?,
            team_invitations: team_invitations::TeamInvitationsRepository::new(db).await?,
            users: users::UsersRepository::new(db).await?,
            votes: votes::VotesRepository::new(db).await?,
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.achievements.ping().await?;
        self.blog_articles.ping().await?;
        self.bots.ping().await?;
        self.bot_stats.ping().await?;
        self.custom_events.ping().await?;
        self.global_stats.ping().await?;
        self.sessions.ping().await?;
        #[cfg(feature = "reports")]
        self.r2.ping().await?;
        #[cfg(feature = "reports")]
        self.stats_reports.ping().await?;
        self.team_invitations.ping().await?;
        self.users.ping().await?;
        self.votes.ping().await?;

        Ok(())
    }
}
