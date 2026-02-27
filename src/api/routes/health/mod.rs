use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};

use crate::{domain::error::ApiResult, openapi::schemas::HealthResponse, repository::Repositories};

#[api_operation(
    summary = "Get API health status",
    description = "Check the health status of the Discord Analytics API",
    tag = "Health"
)]
async fn get_health(repos: Data<Repositories>) -> ApiResult<Json<HealthResponse>> {
    let repos_status = repos.ping().await.is_ok();
    Ok(Json(HealthResponse {
        status: (if repos_status { "healthy" } else { "degraded" }).to_string(),
        service: "Discord Analytics API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: (if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        })
        .to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/health", get().to(get_health));
}
