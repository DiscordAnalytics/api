mod health;

use apistos::web::{ServiceConfig, scope};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("").configure(health::configure));
}
