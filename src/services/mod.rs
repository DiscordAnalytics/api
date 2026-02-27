use crate::repository::Repositories;

mod auth;
mod bots;
mod discord;
mod invitations;
mod users;

#[derive(Clone)]
pub struct Services {
    pub auth: auth::AuthService,
    pub bots: bots::BotsService,
    pub discord: discord::DiscordService,
    pub invitations: invitations::InvitationsService,
    pub users: users::UsersService,
}

impl Services {
    pub fn new(repos: Repositories) -> Self {
        Self {
            auth: auth::AuthService::new(repos.clone()),
            bots: bots::BotsService::new(repos.clone()),
            discord: discord::DiscordService::new(),
            invitations: invitations::InvitationsService::new(repos.clone()),
            users: users::UsersService::new(repos.clone()),
        }
    }
}
