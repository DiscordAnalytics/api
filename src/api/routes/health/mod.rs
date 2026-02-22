use actix_web::web::Json;
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};

use crate::{domain::error::ApiResult, openapi::schemas::HealthResponse};

#[api_operation(
    summary = "Get API health status",
    description = "Check the health status of the Discord Analytics API",
    tag = "Health"
)]
async fn get_health() -> ApiResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
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
