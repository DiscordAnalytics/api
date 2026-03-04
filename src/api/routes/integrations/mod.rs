use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};
use serde_json::{Value, from_slice};

use crate::{
    api::{
        middleware::RawBody,
        routes::integrations::providers::{
            IntegrationResponse, IntegrationResult, handle_provider,
        },
    },
    domain::error::{ApiError, ApiResult},
    repository::Repositories,
};

mod providers;

#[api_operation(
    summary = "Handle incoming webhooks from vote providers",
    description = "This endpoint receives webhooks from various vote providers, processes the payload, and updates the vote counts accordingly. The provider is specified in the URL path, and the payload format may vary based on the provider. The endpoint also verifies the authenticity of the webhook using provider-specific methods to ensure that only legitimate webhooks are processed.",
    tag = "Webhooks"
)]
async fn vote_integration(
    repos: Data<Repositories>,
    raw_body: RawBody,
    path: Path<String>,
) -> ApiResult<Json<IntegrationResult>> {
    let provider = path.into_inner();

    let body_value = match from_slice::<Value>(&raw_body.0) {
        Ok(v) => v,
        Err(e) => {
            return Err(ApiError::InvalidInput(format!(
                "Failed to parse JSON body: {}",
                e
            )));
        }
    };

    match handle_provider(&provider, body_value, repos).await? {
        IntegrationResponse::Accepted(integration_result) => Ok(Json(integration_result)),
        IntegrationResponse::Ignored => Err(ApiError::InvalidInput(format!(
            "Unsupported integration provider: {}",
            provider
        ))),
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/integrations/{provider}", post().to(vote_integration));
}
