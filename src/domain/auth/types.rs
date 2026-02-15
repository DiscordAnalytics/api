use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthType {
    Admin,
    Api,
    Bot,
    Unknown,
    User,
}

impl AuthType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "Api" => AuthType::Api,
            "User" => AuthType::User,
            "Bot" => AuthType::Bot,
            "Admin" => AuthType::Admin,
            _ => AuthType::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthType::Unknown => "Unknown",
            AuthType::Api => "Api",
            AuthType::User => "User",
            AuthType::Bot => "Bot",
            AuthType::Admin => "Admin",
        }
    }
}

impl fmt::Display for AuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
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

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub auth_type: AuthType,
    pub user_id: Option<String>,
    pub bot_id: Option<String>,
}

impl AuthContext {
    pub fn new(auth_type: AuthType) -> Self {
        Self {
            auth_type,
            user_id: None,
            bot_id: None,
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
