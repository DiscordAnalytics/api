use std::fmt::{Display, Formatter, Result};

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
    /// Conflict events (e.g., duplicate entries)
    Conflict,
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
    /// Websocket events
    Websocket,
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
            LogCode::Conflict => "CONFLICT",
            LogCode::Achievement => "ACHV",
            LogCode::Info => "INFO",
            LogCode::System => "SYS",
            LogCode::DbError => "DB_ERR",
            LogCode::Mail => "MAIL",
            LogCode::Websocket => "WS",
        }
    }
}

impl Display for LogCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.as_str())
    }
}
