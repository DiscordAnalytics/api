use std::time::Duration;

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub const MAX_DATE_RANGE: i64 = 367 * 24 * 60 * 60; // 365 days + 2 extra buffer days in seconds

pub const MAX_BOTS_PER_USER: i32 = 3;
pub const MAX_CUSTOM_EVENTS_PER_BOT: i32 = 15;
pub const MAX_GOALS_PER_BOT: i32 = 30;
pub const MAX_TEAMMATES_PER_BOT: i32 = 5;

pub const MAX_WEBHOOK_RETRIES: u8 = 15;

pub const DISCORD_EPOCH: i64 = 1420070400000;

pub const ACCESS_TOKEN_LIFETIME: i64 = 30 * 60; // 30 minutes
pub const REFRESH_TOKEN_LIFETIME: i64 = 30 * 24 * 60 * 60; // 30 days

pub const ACHIEVEMENTS_COLLECTION: &str = "Achievements";
pub const BLOG_ARTICLES_COLLECTION: &str = "BlogArticles";
pub const BOTS_COLLECTION: &str = "Bots";
pub const BOT_STATS_COLLECTION: &str = "BotStats";
pub const CUSTOM_EVENTS_COLLECTION: &str = "CustomEvents";
pub const GLOBAL_STATS_COLLECTION: &str = "GlobalStats";
pub const SESSIONS_COLLECTION: &str = "Sessions";
pub const STATS_REPORTS_COLLECTION: &str = "StatsReports";
pub const TEAM_INVITATIONS_COLLECTION: &str = "TeamInvitations";
pub const USERS_COLLECTION: &str = "Users";
pub const VOTES_COLLECTION: &str = "Votes";
