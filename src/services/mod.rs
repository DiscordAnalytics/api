use crate::repository::Repositories;

mod auth;
mod bots;
mod discord;
mod invitations;
#[cfg(feature = "mails")]
mod mail;
mod users;
mod webhooks;

#[derive(Clone)]
pub struct Services {
    pub auth: auth::AuthService,
    pub bots: bots::BotsService,
    pub discord: discord::DiscordService,
    pub invitations: invitations::InvitationsService,
    #[cfg(feature = "mails")]
    pub mail: mail::MailService,
    pub users: users::UsersService,
    pub webhooks: webhooks::WebhooksService,
}

impl Services {
    pub fn new(repos: Repositories) -> Self {
        Self {
            auth: auth::AuthService::new(repos.clone()),
            bots: bots::BotsService::new(repos.clone()),
            discord: discord::DiscordService::new(),
            invitations: invitations::InvitationsService::new(repos.clone()),
            #[cfg(feature = "mails")]
            mail: mail::MailService::new(),
            users: users::UsersService::new(repos.clone()),
            webhooks: webhooks::WebhooksService::new(repos.clone()),
        }
    }
}
