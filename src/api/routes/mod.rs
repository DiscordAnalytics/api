mod health;

use apistos::web::ServiceConfig;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.configure(health::configure);
}
