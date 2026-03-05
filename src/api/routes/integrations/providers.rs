use actix_web::web::Data;
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, from_value};
use tracing::{info, warn};

use crate::{
    app_env,
    domain::{
        error::{ApiError, ApiResult},
        models::WebhookConfig,
    },
    openapi::schemas::TopGGIntegrationPayload,
    repository::{BotUpdate, Repositories},
    utils::logger::LogCode,
};

#[derive(Serialize, ApiComponent, JsonSchema)]
pub struct IntegrationResult {
    pub webhook_url: String,
    pub routes: Vec<&'static str>,
}

pub enum IntegrationResponse {
    Accepted(IntegrationResult),
    Ignored,
}

pub async fn handle_provider(
    provider: &str,
    body: Value,
    repos: Data<Repositories>,
) -> ApiResult<IntegrationResponse> {
    match provider {
        "topgg" => handle_topgg_integration(body, repos).await,
        _ => Ok(IntegrationResponse::Ignored),
    }
}

async fn handle_topgg_integration(
    body: Value,
    repos: Data<Repositories>,
) -> ApiResult<IntegrationResponse> {
    let payload = match from_value::<TopGGIntegrationPayload>(body) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                error = %e,
                "Failed to parse TopGG integration payload"
            );
            return Err(ApiError::InvalidInput("Invalid TopGG payload".to_string()));
        }
    };

    if payload.type_ == "integration.delete" {
        return Ok(IntegrationResponse::Ignored);
    }

    let project = payload.data.project.ok_or_else(|| {
        warn!(
            code = %LogCode::Webhook,
            provider = "topgg",
            "Received TopGG integration payload without project information"
        );
        ApiError::InvalidInput("Missing project information in TopGG payload".to_string())
    })?;

    if project.platform != "discord" {
        warn!(
            code = %LogCode::Webhook,
            provider = "topgg",
            platform = %project.platform,
            "Received TopGG integration for unsupported platform"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    if project.type_ != "bot" {
        warn!(
            code = %LogCode::Webhook,
            provider = "topgg",
            project_type = %project.type_,
            "Received TopGG integration for unsupported project type"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    if payload.type_ != "integration.create" {
        warn!(
            code = %LogCode::Webhook,
            provider = "topgg",
            event_type = %payload.type_,
            "Received unsupported TopGG integration event type"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    let update = BotUpdate::new().with_webhook_config(
        "topgg",
        WebhookConfig {
            connection_id: Some(payload.data.connection_id),
            webhook_url: None,
            webhook_secret: payload.data.webhook_secret,
        },
    );

    repos.bots.update(&project.platform_id, update).await?;

    info!(
        code = %LogCode::Webhook,
        provider = "topgg",
        bot_id = %project.platform_id,
        "Successfully processed TopGG integration event"
    );

    Ok(IntegrationResponse::Accepted(IntegrationResult {
        webhook_url: format!("{}/webhooks/topgg", app_env!().api_url),
        routes: vec!["vote.create"],
    }))
}
