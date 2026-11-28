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
pub const SESSIONS_COLLECTION: &str = "Sessions";
#[cfg(feature = "reports")]
pub const STATS_REPORTS_COLLECTION: &str = "StatsReports";
pub const STATUS_CHECKS_COLLECTION: &str = "StatusChecks";
pub const STATUS_CHECKS_HOURLY_COLLECTION: &str = "StatusChecksHourly";
pub const STATUS_CURRENT_COLLECTION: &str = "StatusCurrent";
pub const STATUS_INCIDENTS_COLLECTION: &str = "StatusIncidents";
pub const STATUS_PENDING_BUCKETS_COLLECTION: &str = "StatusPendingBuckets";
pub const TEAM_INVITATIONS_COLLECTION: &str = "TeamInvitations";
pub const USERS_COLLECTION: &str = "Users";
pub const VOTES_COLLECTION: &str = "Votes";

/// Raw check TTL: 7 days.
pub const STATUS_CHECK_TTL_SECS: u64 = 7 * 24 * 60 * 60;
/// Hourly check TTL: 90 days.
pub const STATUS_CHECK_HOURLY_TTL_SECS: u64 = 90 * 24 * 60 * 60;
/// Pending bucket TTL: 1 hour (safety net - sealed long before this).
pub const STATUS_PENDING_BUCKET_TTL_SECS: u64 = 7 * 24 * 60 * 60;
/// Heartbeat interval the SDK is expected to use.
pub const STATUS_HEARTBEAT_INTERVAL_SECS: i64 = 5 * 60;
/// Bucket window size matches the heartbeat interval.
pub const STATUS_BUCKET_WINDOW_SECS: i64 = STATUS_HEARTBEAT_INTERVAL_SECS;
pub const MAX_SEAL_BATCH_SIZE: i64 = 100;
