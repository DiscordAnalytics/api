use std::time::Duration;

pub const PUBLIC_ROUTES: [&str; 2] = ["/articles", "/articles/{id}"];

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub const MAX_BOTS_PER_USER: i32 = 3;

pub const MAX_WEBHOOK_RETRIES: u8 = 15;

pub const TAG_LEN: usize = 16;

pub const DISCORD_EPOCH: i64 = 1420070400000;

pub const ACCESS_TOKEN_LIFETIME: i64 = 30 * 60; // 30 minutes
pub const REFRESH_TOKEN_LIFETIME: i64 = 30 * 24 * 60 * 60; // 30 days

#[cfg(debug_assertions)]
pub const DB_NAME: &str = "api-dev";
#[cfg(not(debug_assertions))]
pub const DB_NAME: &str = "api";
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
