mod achievements;
mod blog_articles;
mod bot_stats;
mod bots;
mod connection;
mod custom_events;
mod global_stats;
mod stats_reports;
mod team_invitations;
mod users;
mod votes;

#[derive(Clone)]
pub struct Repositories {
    pub achievements: achievements::AchievementsRepository,
    pub blog_articles: blog_articles::BlogArticlesRepository,
    pub bots: bots::BotsRepository,
    pub bot_stats: bot_stats::BotStatsRepository,
    pub custom_events: custom_events::CustomEventsRepository,
    pub global_stats: global_stats::GlobalStatsRepository,
    pub stats_reports: stats_reports::StatsReportsRepository,
    pub team_invitations: team_invitations::TeamInvitationsRepository,
    pub users: users::UsersRepository,
    pub votes: votes::VotesRepository,
}

impl Repositories {
    pub async fn init() -> mongodb::error::Result<Self> {
        let connection = connection::DbConnection::init().await?;
        let db = connection.database();

        Ok(Self {
            achievements: achievements::AchievementsRepository::new(db),
            blog_articles: blog_articles::BlogArticlesRepository::new(db),
            bots: bots::BotsRepository::new(db),
            bot_stats: bot_stats::BotStatsRepository::new(db),
            custom_events: custom_events::CustomEventsRepository::new(db),
            global_stats: global_stats::GlobalStatsRepository::new(db),
            stats_reports: stats_reports::StatsReportsRepository::new(db),
            team_invitations: team_invitations::TeamInvitationsRepository::new(db),
            users: users::UsersRepository::new(db),
            votes: votes::VotesRepository::new(db),
        })
    }
}
