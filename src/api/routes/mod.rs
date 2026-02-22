mod achievements;
mod bots;
mod health;

use apistos::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(bots::configure)
        .configure(health::configure);
}
