use anyhow::Result;

use crate::repository::Repositories;

#[derive(Clone)]
pub struct InvitationsService {
    repos: Repositories,
}

impl InvitationsService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn reject_invitation(
        &self,
        invitation_id: &str,
        bot_id: &str,
        user_id: &str,
    ) -> Result<()> {
        self.repos
            .team_invitations
            .delete_by_id(invitation_id)
            .await?;
        self.repos
            .bots
            .remove_user_from_team(bot_id, user_id)
            .await?;
        Ok(())
    }
}
