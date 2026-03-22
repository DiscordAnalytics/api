mod achievement;
mod article;
mod auth;
mod bot;
mod bot_stat;
mod custom_event;
mod health;
mod integrations;
mod invitation;
mod session;
mod team;
mod user;
mod vote;
mod webhook;

use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use achievement::{
    AchievementCreationPayload, AchievementResponse, AchievementUpdatePayload,
    DeleteAchievementQuery,
};
pub use article::{ArticleAuthor, ArticleDeleteResponse, ArticleRequest, ArticleResponse};
pub use auth::{
    AuthCallbackQuery, AuthConfigResponse, DiscordBot, DiscordOAuthUser, DiscordTokenResponse,
    LinkedRolesQuery,
};
pub use bot::{
    BotCreationBody, BotDeletionPayload, BotResponse, BotSettingsPayload, BotSuspendRequest,
    BotTokenResponse, BotUpdateBody,
};
pub use bot_stat::{
    BotStatsBody, BotStatsContent, BotStatsQuery, BotStatsResponse, NormalizedStatsBody,
};
pub use custom_event::{CustomEventBody, CustomEventResponse, CustomEventUpdatePayload};
pub use health::HealthResponse;
pub use integrations::{IntegrationPayload, TopGGIntegrationPayload};
pub use invitation::{
    InvitationAcceptBody, InvitationAcceptResponse, InvitationResponse, TeamInvitationResponse,
};
pub use session::{RefreshTokenRequest, SessionResponse, TokenResponse};
pub use team::{NewInvitationResponse, TeamRequestBody, TeamResponse};
pub use user::{UserBotsResponse, UserResponse, UserSuspendRequest, UserUpdateRequest};
pub use vote::VoteResponse;
pub use webhook::{
    BotListMePayload, DBListPayload, DiscordListPayload, DiscordPlacePayload, DiscordsComPayload,
    TopGGPayload,
};

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct MessageResponse {
    pub message: String,
}
