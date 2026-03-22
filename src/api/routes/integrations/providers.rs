use actix_web::web::Data;
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, from_value};
use tracing::{info, warn};

use crate::{
    app_env,
    domain::{
        auth::generate_bot_token,
        error::{ApiError, ApiResult},
        models::{Bot, WebhookConfig},
    },
    openapi::schemas::{IntegrationPayload, TopGGIntegrationPayload},
    repository::{BotUpdate, Repositories},
    services::Services,
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
    services: Data<Services>,
    repos: Data<Repositories>,
) -> ApiResult<IntegrationResponse> {
    match provider {
        "topgg" => handle_topgg_integration(body, services, repos).await,
        "botlistme" | "dblist" | "discordlist" | "discordplace" | "discordscom" => {
            handle_integration(provider, body, services, repos).await
        }
        _ => Ok(IntegrationResponse::Ignored),
    }
}

async fn handle_integration(
    provider: &str,
    body: Value,
    services: Data<Services>,
    repos: Data<Repositories>,
) -> ApiResult<IntegrationResponse> {
    let payload = match from_value::<IntegrationPayload>(body) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                code = %LogCode::Webhook,
                provider = %provider,
                error = %e,
                "Failed to parse integration payload"
            );
            return Err(ApiError::InvalidInput(
                "Invalid integration payload".to_string(),
            ));
        }
    };

    let bot_id = &payload.bot_id;
    if repos.bots.find_by_id(bot_id).await?.is_none() {
        let token = generate_bot_token(bot_id).map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                error = %e,
                "Failed to generate bot token for new integration"
            );
            ApiError::InternalError("Failed to generate bot token".to_string())
        })?;
        let bot_details = services.discord.get_bot(bot_id).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                error = %e,
                "Failed to fetch bot details from Discord for new integration"
            );
            ApiError::InternalError("Failed to fetch bot details".to_string())
        })?;
        if let Some(is_bot) = bot_details.bot
            && !is_bot
        {
            return Ok(IntegrationResponse::Ignored);
        }
        let new_bot = Bot::new(
            bot_id,
            &payload.user_id,
            token,
            &bot_details.username,
            bot_details.avatar.as_deref(),
        );
        repos.bots.insert(&new_bot).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                bot_id = %bot_id,
                error = %e,
                "Failed to insert new bot from integration into database"
            );
            ApiError::InternalError("Failed to create bot".to_string())
        })?;
    }

    let update = BotUpdate::new().with_webhook_config(
        provider,
        WebhookConfig {
            connection_id: None,
            webhook_secret: payload.webhook_secret,
        },
        None,
    );

    repos.bots.update(bot_id, update).await?;

    info!(
        code = %LogCode::Webhook,
        provider = %provider,
        bot_id = %bot_id,
        "Successfully processed integration event"
    );

    Ok(IntegrationResponse::Accepted(IntegrationResult {
        webhook_url: format!("{}/webhooks/{}", app_env!().api_url, provider),
        routes: vec![],
    }))
}

async fn handle_topgg_integration(
    body: Value,
    services: Data<Services>,
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

    if repos.bots.find_by_id(&project.platform_id).await?.is_none() {
        let bot_id = &project.platform_id;
        let user = payload.data.user.ok_or_else(|| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                "Received TopGG integration payload without user information for new bot"
            );
            ApiError::InvalidInput("Missing user information in TopGG payload".to_string())
        })?;
        let token = generate_bot_token(bot_id).map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                bot_id = %bot_id,
                error = %e,
                "Failed to generate bot token for new TopGG integration"
            );
            ApiError::InternalError("Failed to generate bot token".to_string())
        })?;
        let bot_details = services.discord.get_bot(bot_id).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                bot_id = %bot_id,
                error = %e,
                "Failed to fetch bot details from Discord for new TopGG integration"
            );
            ApiError::InternalError("Failed to fetch bot details".to_string())
        })?;
        if let Some(is_bot) = bot_details.bot
            && !is_bot
        {
            return Ok(IntegrationResponse::Ignored);
        }
        let new_bot = Bot::new(
            bot_id,
            &user.platform_id,
            token,
            &bot_details.username,
            bot_details.avatar.as_deref(),
        );
        repos.bots.insert(&new_bot).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = "topgg",
                bot_id = %bot_id,
                error = %e,
                "Failed to insert new bot from TopGG integration into database"
            );
            ApiError::InternalError("Failed to create bot".to_string())
        })?;
    }

    let update = BotUpdate::new().with_webhook_config(
        "topgg",
        WebhookConfig {
            connection_id: Some(payload.data.connection_id),
            webhook_secret: payload.data.webhook_secret,
        },
        None,
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
