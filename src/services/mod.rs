use crate::repository::Repositories;

mod auth;
mod bots;
mod discord;
mod invitations;
mod users;
mod webhooks;

#[derive(Clone)]
pub struct Services {
    pub auth: auth::AuthService,
    pub bots: bots::BotsService,
    pub discord: discord::DiscordService,
    pub invitations: invitations::InvitationsService,
    pub users: users::UsersService,
    pub webhooks: webhooks::WebhooksService,
}

impl Services {
    pub fn new(repos: Repositories) -> Self {
        let bots_service = bots::BotsService::new(repos.clone());

        Self {
            auth: auth::AuthService::new(repos.clone()),
            bots: bots_service.clone(),
            discord: discord::DiscordService::new(),
            invitations: invitations::InvitationsService::new(repos.clone()),
            users: users::UsersService::new(repos.clone(), &bots_service),
            webhooks: webhooks::WebhooksService::new(repos.clone()),
        }
    }
}
