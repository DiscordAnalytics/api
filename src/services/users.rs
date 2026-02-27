use anyhow::Result;

use crate::repository::Repositories;

#[derive(Clone)]
pub struct UsersService {
    repos: Repositories,
}

impl UsersService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn has_reached_bots_limit(&self, user_id: &str) -> Result<bool> {
        let user_details = self
            .repos
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;
        let bot_count = self.repos.bots.count_by_user_id(user_id).await?;
        Ok((bot_count as i32) >= user_details.bots_limit)
    }

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.repos.users.delete_by_id(user_id).await?;
        self.repos.sessions.revoke_all_for_user(user_id).await?;
        let user_bots = self.repos.bots.find_by_owner_id(user_id).await?;
        for bot in user_bots {
            self.repos.bots.delete(&bot.bot_id).await?;
            self.repos.bot_stats.delete_by_bot_id(&bot.bot_id).await?;
            self.repos.votes.delete_by_bot_id(&bot.bot_id).await?;
            self.repos
                .achievements
                .delete_by_bot_id(&bot.bot_id)
                .await?;
            self.repos
                .stats_reports
                .delete_by_bot_id(&bot.bot_id)
                .await?;
            self.repos
                .team_invitations
                .delete_by_bot_id(&bot.bot_id)
                .await?;
            self.repos
                .custom_events
                .delete_by_bot_id(&bot.bot_id)
                .await?;
        }
        self.repos.stats_reports.delete_by_user_id(user_id).await?;
        self.repos
            .team_invitations
            .delete_by_user_id(user_id)
            .await?;

        Ok(())
    }
}
