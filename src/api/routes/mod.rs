mod achievements;
mod articles;
mod auth;
mod bots;
mod health;
mod invitations;
mod stats;
mod users;
mod websocket;

use apistos::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(articles::configure)
        .configure(auth::configure)
        .configure(bots::configure)
        .configure(health::configure)
        .configure(invitations::configure)
        .configure(stats::configure)
        .configure(users::configure)
        .configure(websocket::configure);
}
