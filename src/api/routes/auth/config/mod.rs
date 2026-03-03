use actix_web::web::Json;
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};

use crate::{app_env, domain::error::ApiResult, openapi::schemas::AuthConfigResponse};

#[api_operation(
    summary = "Get authentication configuration",
    description = "Returns necessary configuration for client-side authentication, such as the OAuth client ID.",
    tag = "Auth"
)]
async fn get_auth_config() -> ApiResult<Json<AuthConfigResponse>> {
    let auth_scopes = [
        "identify",
        #[cfg(feature = "mails")]
        "email",
    ];

    Ok(Json(AuthConfigResponse {
        client_id: app_env!().client_id.clone(),
        scopes: auth_scopes.iter().map(|s| s.to_string()).collect(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/config", get().to(get_auth_config));
}
