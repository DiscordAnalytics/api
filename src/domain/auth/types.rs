use std::fmt;

use apistos::ApiComponent;
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, ApiComponent)]
pub enum AuthType {
    Admin,
    Bot,
    Unknown,
    User,
}

impl AuthType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Admin" => AuthType::Admin,
            "Bot" => AuthType::Bot,
            "User" => AuthType::User,
            _ => AuthType::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::Admin => "Admin",
            AuthType::Bot => "Bot",
            AuthType::User => "User",
            _ => "Unknown",
        }
    }
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Authorization {
    pub auth_type: AuthType,
    pub token: String,
}

impl Authorization {
    pub fn parse(header_value: &str) -> Option<Self> {
        let parts: Vec<&str> = header_value.split_whitespace().collect();

        if parts.len() != 2 {
            return None;
        }

        Some(Self {
            auth_type: AuthType::from_str(parts[0]),
            token: parts[1].to_string(),
        })
    }
}

#[derive(Debug, Clone, JsonSchema, ApiComponent)]
pub struct AuthContext {
    pub auth_type: AuthType,
    pub user_id: Option<String>,
    pub bot_id: Option<String>,
    pub session_id: Option<String>,
    pub token: Option<String>,
}

impl AuthContext {
    pub fn new(auth_type: AuthType) -> Self {
        Self {
            auth_type,
            user_id: None,
            bot_id: None,
            session_id: None,
            token: None,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_bot_id(mut self, bot_id: String) -> Self {
        self.bot_id = Some(bot_id);
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn is_admin(&self) -> bool {
        self.auth_type == AuthType::Admin
    }

    pub fn is_bot(&self) -> bool {
        self.auth_type == AuthType::Bot
    }

    pub fn is_user(&self) -> bool {
        matches!(self.auth_type, AuthType::User | AuthType::Admin)
    }
}
