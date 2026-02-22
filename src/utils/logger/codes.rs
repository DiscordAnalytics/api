use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCode {
    /// Server startup/shutdown events
    Server,
    /// HTTP request logging
    Request,
    /// Database operations
    Database,
    /// Authentication/Authorization events
    Auth,
    /// Unauthorized access attempts
    Unauthorized,
    /// Forbidden access attempts
    Forbidden,
    /// Invalid token or authentication failures
    InvalidToken,
    /// Admin actions
    AdminAction,
    /// Bot-related events
    Bot,
    /// Bot expiration warnings
    BotExpiration,
    /// User-related events
    User,
    /// Achievement events
    Achievement,
    /// General information
    Info,
    /// System/Internal errors
    System,
    /// Database errors
    DbError,
    /// Mail-related events
    Mail,
}

impl LogCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCode::Server => "SERVER",
            LogCode::Request => "REQ",
            LogCode::Database => "DB",
            LogCode::Auth => "AUTH",
            LogCode::Unauthorized => "UNAUTH",
            LogCode::Forbidden => "FORBID",
            LogCode::InvalidToken => "INV_TOKEN",
            LogCode::AdminAction => "ADMIN",
            LogCode::Bot => "BOT",
            LogCode::BotExpiration => "BOT_EXP",
            LogCode::User => "USER",
            LogCode::Achievement => "ACHV",
            LogCode::Info => "INFO",
            LogCode::System => "SYS",
            LogCode::DbError => "DB_ERR",
            LogCode::Mail => "MAIL",
        }
    }
}

impl fmt::Display for LogCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
