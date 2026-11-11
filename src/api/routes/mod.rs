mod achievements;
mod articles;
mod auth;
mod bots;
mod health;
mod integrations;
mod invitations;
mod users;
mod webhooks;
mod websocket;

use apistos::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(articles::configure)
        .configure(auth::configure)
        .configure(bots::configure)
        .configure(health::configure)
        .configure(integrations::configure)
        .configure(invitations::configure)
        .configure(users::configure)
        .configure(webhooks::configure)
        .configure(websocket::configure);
}
