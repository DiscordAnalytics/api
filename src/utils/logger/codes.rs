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
    /// Get the string representation for log formatting
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCode::Server => "SERVER",
            LogCode::Request => "REQ",
            LogCode::Database => "DB",
            LogCode::Auth => "AUTH",
            LogCode::Unauthorized => "UNAUTH",
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

    /// Get color for terminal output (optional, for dev mode)
    #[cfg(feature = "colored-logs")]
    pub fn color(&self) -> colored::Color {
        use fern::colors::Color;
        match self {
            LogCode::Server => Color::Blue,
            LogCode::Request => Color::Cyan,
            LogCode::Database => Color::Magenta,
            LogCode::Auth => Color::Green,
            LogCode::Unauthorized => Color::Yellow,
            LogCode::AdminAction => Color::BrightMagenta,
            LogCode::Bot => Color::BrightBlue,
            LogCode::BotExpiration => Color::BrightYellow,
            LogCode::User => Color::BrightCyan,
            LogCode::Achievement => Color::BrightGreen,
            LogCode::Info => Color::White,
            LogCode::System => Color::BrightRed,
            LogCode::DbError => Color::Red,
            LogCode::Mail => Color::BrightWhite,
        }
    }
}

impl fmt::Display for LogCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
