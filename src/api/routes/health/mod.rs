use actix_web::web::Json;
use apistos::{
    ApiComponent, api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::error::ApiResult;

#[derive(Deserialize, Serialize, Clone, ApiComponent, JsonSchema)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    environment: String,
}

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
    cfg.service(scope("/health").service(resource("").route(get().to(get_health))));
}
