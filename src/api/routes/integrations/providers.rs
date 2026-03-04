use actix_web::web::Data;
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, from_value};
use tracing::warn;

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
    pub routes: Vec<String>,
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

    let project = payload.data.project;

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

    repos
        .bots
        .find_by_id(&project.platform_id)
        .await?
        .ok_or_else(|| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                bot_id = %project.platform_id,
                "Received TopGG integration for non-existent bot"
            );
            ApiError::NotFound("Bot not found".to_string())
        })?;

    if payload.type_ == "integration.create" {
        let update = BotUpdate::new().with_webhook_config(
            "topgg",
            WebhookConfig {
                connection_id: Some(payload.data.connection_id),
                webhook_url: None,
                webhook_secret: Some(payload.data.webhook_secret),
            },
        );

        repos.bots.update(&project.platform_id, update).await?;
    } else if payload.type_ == "integration.delete" {
        let update = BotUpdate::new().with_webhook_config(
            "topgg",
            WebhookConfig {
                connection_id: None,
                webhook_url: None,
                webhook_secret: None,
            },
        );

        repos.bots.update(&project.platform_id, update).await?;
    } else {
        warn!(
            code = %LogCode::Webhook,
            provider = "topgg",
            event_type = %payload.type_,
            "Received unsupported TopGG integration event type"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    Ok(IntegrationResponse::Accepted(IntegrationResult {
        webhook_url: format!("{}/webhooks/topgg", app_env!().api_url),
        routes: vec!["vote.create".to_string(), "webhook.test".to_string()],
    }))
}
