use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::models::TeamInvitation;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationAcceptBody {
    pub accept: bool,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    pub invitation: TeamInvitationResponse,
    pub bot_username: String,
    pub bot_avatar: Option<String>,
    pub user_username: String,
    pub user_avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserInvitationResponse {
    pub invitation: TeamInvitationResponse,
    pub bot_username: String,
    pub bot_avatar: Option<String>,
    pub owner_username: String,
    pub owner_avatar: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct InvitationAcceptResponse {
    pub accepted: bool,
}

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TeamInvitationResponse {
    accepted: bool,
    bot_id: String,
    expiration: String,
    invitation_id: String,
    user_id: String,
}

impl TryFrom<TeamInvitation> for TeamInvitationResponse {
    type Error = anyhow::Error;

    fn try_from(invitation: TeamInvitation) -> Result<Self, Self::Error> {
        Ok(Self {
            accepted: invitation.accepted,
            bot_id: invitation.bot_id,
            expiration: invitation.expiration.try_to_rfc3339_string()?,
            invitation_id: invitation.invitation_id,
            user_id: invitation.user_id,
        })
    }
}
