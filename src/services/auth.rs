use anyhow::{Result, bail};

use crate::{
    app_env,
    domain::auth::{AuthContext, AuthType},
    repository::Repositories,
};

#[derive(Clone)]
pub struct AuthService {
    repos: Repositories,
}

impl AuthService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub fn verify_api_token(&self, token: &str) -> Result<AuthContext> {
        if token == app_env!().admin_token.as_str() {
            Ok(AuthContext::new(AuthType::Api))
        } else {
            bail!("Invalid API token");
        }
    }

    pub async fn verify_bot_token(&self, token: &str, bot_id: &str) -> Result<AuthContext> {
        let bot = self.repos.bots.find_by_id(bot_id).await?;
        match bot {
            Some(bot) if bot.token == token => {
                Ok(AuthContext::new(AuthType::Bot).with_bot_id(bot_id.to_string()))
            }
            Some(_) => bail!("Invalid bot token"),
            None => bail!("Bot not found"),
        }
    }

    pub async fn verify_user_token(&self, token: &str) -> Result<AuthContext> {
        let user = self.repos.users.find_by_token(token).await?;
        match user {
            Some(user) => {
                let auth_type = if self.is_admin(&user.user_id) {
                    AuthType::Admin
                } else {
                    AuthType::User
                };
                Ok(AuthContext::new(auth_type).with_user_id(user.user_id))
            }
            None => bail!("User not found or invalid token"),
        }
    }

    pub fn is_admin(&self, user_id: &str) -> bool {
        app_env!().admins.iter().any(|admin_id| admin_id == user_id)
    }

    pub async fn user_has_bot_access(&self, user_id: &str, bot_id: &str) -> Result<bool> {
        let bot = self.repos.bots.find_by_id(bot_id).await?;
        match bot {
            Some(bot) => Ok(bot.has_access(user_id)),
            None => Ok(false),
        }
    }

    pub async fn user_owns_bot(&self, user_id: &str, bot_id: &str) -> Result<bool> {
        let bot = self.repos.bots.find_by_id(bot_id).await?;
        match bot {
            Some(bot) => Ok(bot.is_owner(user_id)),
            None => Ok(false),
        }
    }
}
