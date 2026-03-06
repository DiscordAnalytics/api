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
mod stat;
mod team;
mod user;
mod vote;
mod webhook;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
pub struct MessageResponse {
    pub message: String,
}

pub use achievement::AchievementResponse;
use apistos::ApiComponent;
pub use article::{ArticleDeleteResponse, ArticleRequest, ArticleResponse};
pub use auth::{
    AuthCallbackQuery, AuthConfigResponse, AvatarDecoration, DiscordOAuthUser,
    DiscordTokenResponse, LinkedRolesQuery,
};
pub use bot::{
    BotCreationBody, BotResponse, BotSettingsPayload, BotSuspendRequest, BotTokenResponse,
    BotUpdateBody,
};
pub use bot_stat::{
    BotStatsBody, BotStatsContent, BotStatsQuery, BotStatsResponse, NormalizedStatsBody,
};
pub use custom_event::{CustomEventBody, CustomEventResponse};
pub use health::HealthResponse;
pub use integrations::TopGGIntegrationPayload;
pub use invitation::{
    InvitationAcceptBody, InvitationAcceptResponse, InvitationResponse, TeamInvitationResponse,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use session::{RefreshTokenRequest, SessionResponse, TokenResponse};
pub use stat::{StatResponse, StatsQuery};
pub use team::{TeamRequestBody, TeamResponse};
pub use user::{UserBotsResponse, UserResponse, UserSuspendRequest, UserUpdateRequest};
pub use vote::VoteResponse;
pub use webhook::{
    BotListMePayload, DBListPayload, DiscordListPayload, DiscordPlacePayload, DiscordsComPayload,
    TopGGPayload, WebhookVoteResponse,
};
