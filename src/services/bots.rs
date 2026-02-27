use anyhow::Result;

use crate::repository::Repositories;

#[derive(Clone)]
pub struct BotsService {
    repos: Repositories,
}

impl BotsService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn delete_bot(&self, bot_id: &str) -> Result<()> {
        self.repos.bots.delete(bot_id).await?;
        self.repos.bot_stats.delete_by_bot_id(bot_id).await?;
        self.repos.votes.delete_by_bot_id(bot_id).await?;
        self.repos.achievements.delete_by_bot_id(bot_id).await?;
        self.repos.team_invitations.delete_by_bot_id(bot_id).await?;
        #[cfg(feature = "reports")]
        self.repos.stats_reports.delete_by_bot_id(bot_id).await?;
        self.repos.custom_events.delete_by_bot_id(bot_id).await?;

        Ok(())
    }
}
