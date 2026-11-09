mod incidents;

use apistos::web::{ServiceConfig, scope};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("/manage").configure(incidents::configure));
}
