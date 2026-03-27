use anyhow::Result;

use crate::{app_env, repository::Repositories};

#[derive(Clone)]
pub struct AuthService {
    repos: Repositories,
}

impl AuthService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
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
}
