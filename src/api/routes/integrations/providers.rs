use actix_web::web::Data;
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, from_value};

use crate::{
    app_env,
    domain::{
        auth::generate_bot_token,
        error::{ApiError, ApiResult},
        models::{Bot, PlatformProvider, WebhookConfig},
    },
    openapi::schemas::{IntegrationPayload, PlatformIntegrationPayload},
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
    if let Some(spec) = PlatformProvider::from_key(provider) {
        return handle_platform_integration(&spec, body, services, repos).await;
    }

    match provider {
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
                provider = %provider,
                bot_id = %bot_id,
                error = %e,
                "Failed to insert new bot from integration into database"
            );
            ApiError::InternalError("Failed to create bot".to_string())
        })?;
    }

    let update = BotUpdate::default().with_webhook_config(
        provider,
        WebhookConfig {
            connection_id: None,
            webhook_secret: payload.webhook_secret,
        },
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

async fn handle_platform_integration(
    spec: &PlatformProvider,
    body: Value,
    services: Data<Services>,
    repos: Data<Repositories>,
) -> ApiResult<IntegrationResponse> {
    let payload = match from_value::<PlatformIntegrationPayload>(body) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                code = %LogCode::Webhook,
                provider = %spec.key,
                error = %e,
                "Failed to parse integration payload"
            );
            return Err(ApiError::InvalidInput(format!(
                "Invalid {} payload",
                spec.key
            )));
        }
    };

    if payload.type_ == "integration.delete"
        && let Some(connection_id) = payload.data.connection_id
    {
        repos
            .bots
            .remove_integration(spec.key, &connection_id)
            .await?;

        return Ok(IntegrationResponse::Ignored);
    }

    let project = payload.data.project.ok_or_else(|| {
        warn!(
            code = %LogCode::Webhook,
            provider = %spec.key,
            "Received integration payload without project information"
        );
        ApiError::InvalidInput(format!(
            "Missing project information in {} payload",
            spec.key
        ))
    })?;

    if project.platform != "discord" {
        warn!(
            code = %LogCode::Webhook,
            provider = %spec.key,
            platform = %project.platform,
            "Received integration for unsupported platform"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    if project.type_ != "bot" {
        warn!(
            code = %LogCode::Webhook,
            provider = %spec.key,
            project_type = %project.type_,
            "Received integration for unsupported project type"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    if payload.type_ != "integration.create" {
        warn!(
            code = %LogCode::Webhook,
            provider = %spec.key,
            event_type = %payload.type_,
            "Received unsupported integration event type"
        );
        return Ok(IntegrationResponse::Ignored);
    }

    if repos.bots.find_by_id(&project.platform_id).await?.is_none() {
        let bot_id = &project.platform_id;
        let user = payload.data.user.ok_or_else(|| {
            warn!(
                code = %LogCode::Webhook,
                provider = %spec.key,
                "Received integration payload without user information for new bot"
            );
            ApiError::InvalidInput(format!("Missing user information in {} payload", spec.key))
        })?;
        let token = generate_bot_token(bot_id).map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = %spec.key,
                bot_id = %bot_id,
                error = %e,
                "Failed to generate bot token for new integration"
            );
            ApiError::InternalError("Failed to generate bot token".to_string())
        })?;
        let bot_details = services.discord.get_bot(bot_id).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = %spec.key,
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
            &user.platform_id,
            token,
            &bot_details.username,
            bot_details.avatar.as_deref(),
        );
        repos.bots.insert(&new_bot).await.map_err(|e| {
            warn!(
                code = %LogCode::Webhook,
                provider = %spec.key,
                bot_id = %bot_id,
                error = %e,
                "Failed to insert new bot from integration into database"
            );
            ApiError::InternalError("Failed to create bot".to_string())
        })?;
    }

    let update = BotUpdate::default().with_webhook_config(
        spec.key,
        WebhookConfig {
            connection_id: payload.data.connection_id,
            webhook_secret: payload.data.webhook_secret,
        },
    );

    repos.bots.update(&project.platform_id, update).await?;

    info!(
        code = %LogCode::Webhook,
        provider = %spec.key,
        bot_id = %project.platform_id,
        "Successfully processed integration event"
    );

    Ok(IntegrationResponse::Accepted(IntegrationResult {
        webhook_url: format!("{}/webhooks/{}", app_env!().api_url, spec.key),
        routes: spec.routes.to_vec(),
    }))
}
