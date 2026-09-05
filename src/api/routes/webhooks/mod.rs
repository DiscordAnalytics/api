mod providers;

use std::sync::Arc;

use actix_web::{
    HttpRequest,
    web::{Data, Json, Path},
};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};
use serde_json::{Value, from_slice};
use tokio::sync::Mutex;

use self::providers::{ProviderResponse, handle_provider};

use crate::{
    api::{middleware::RawBody, routes::webhooks::providers::get_provider_info},
    domain::error::{ApiError, ApiResult},
    managers::VotesWebhooksManager,
    openapi::schemas::MessageResponse,
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

fn extract_bot_id_from_payload(provider: &str, body: &Value) -> Option<String> {
    match provider {
        "topgg" | "botillon" => body
            .get("data")?
            .get("project")?
            .get("platform_id")?
            .as_str()
            .map(String::from),
        "dblist" | "discordlist" => body.get("bot_id")?.as_str().map(String::from),
        "discordscom" | "botlistme" | "discordplace" => body.get("bot")?.as_str().map(String::from),
        "test" => body.get("botId")?.as_str().map(String::from),
        _ => None,
    }
}

#[api_operation(
    summary = "Handle incoming webhooks from vote providers",
    description = "This endpoint receives webhooks from various vote providers, processes the payload, and updates the vote counts accordingly. The provider is specified in the URL path, and the payload format may vary based on the provider. The endpoint also verifies the authenticity of the webhook using provider-specific methods to ensure that only legitimate webhooks are processed.",
    tag = "Webhooks"
)]
async fn vote_webhook(
    req: HttpRequest,
    services: Data<Services>,
    repos: Data<Repositories>,
    webhook_manager: Data<Arc<Mutex<VotesWebhooksManager>>>,
    path: Path<String>,
    body: RawBody,
) -> ApiResult<Json<MessageResponse>> {
    let provider = path.into_inner();

    let body_bytes = &body.into_inner();

    let body_value = from_slice::<Value>(body_bytes).map_err(|e| {
        warn!(
            code = %LogCode::Webhook,
            provider = %provider,
            error = %e,
            "Failed to parse JSON body in webhook"
        );
        ApiError::WebhookError("Invalid JSON body".to_string())
    })?;

    info!(
        code = %LogCode::Webhook,
        provider = %provider,
        body = ?body_value,
        "Received webhook with body"
    );

    let bot_id = extract_bot_id_from_payload(&provider, &body_value).ok_or_else(|| {
        warn!(
            code = %LogCode::Webhook,
            provider = %provider,
            "Failed to extract bot ID from webhook payload"
        );
        ApiError::WebhookError("Missing bot ID in payload".to_string())
    })?;

    let headers = req.headers();

    let authorization = headers.get("Authorization").and_then(|h| h.to_str().ok());

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        warn!(
            code = %LogCode::Webhook,
            provider = %provider,
            bot_id = %bot_id,
            "Received webhook for non-existent bot"
        );
        ApiError::NotFound("Bot not found".to_string())
    })?;

    let response = handle_provider(
        &provider,
        body_value.clone(),
        body_bytes,
        authorization,
        &bot,
        headers,
    )
    .await?;

    match response {
        ProviderResponse::Vote(vote_result) => {
            if provider != "test" {
                services
                    .webhooks
                    .record_vote(
                        &bot_id,
                        &vote_result.voter_id,
                        &provider,
                        vote_result.vote_count,
                    )
                    .await?;

                services
                    .webhooks
                    .trigger_webhook_notification(
                        &bot,
                        &vote_result.voter_id,
                        &provider,
                        body_value,
                        &webhook_manager,
                    )
                    .await?;
            }

            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                voter_id = %vote_result.voter_id,
                vote_count = vote_result.vote_count,
                "Processed vote webhook successfully"
            );

            Ok(Json(MessageResponse {
                message: "Vote processed successfully".to_string(),
            }))
        }
        ProviderResponse::TestWebhook => {
            services
                .webhooks
                .trigger_webhook_notification(&bot, "0", &provider, body_value, &webhook_manager)
                .await?;

            if let Some(owner) = repos.users.find_by_id(&bot.owner_id).await? {
                let provider_info = get_provider_info(&provider).ok_or_else(|| {
                    warn!(
                        code = %LogCode::Webhook,
                        provider = %provider,
                        "Unknown provider for test webhook email"
                    );
                    ApiError::WebhookError("Unknown provider".to_string())
                })?;

                services
                    .webhooks
                    .send_test_webhook_email(
                        &bot,
                        &owner,
                        &provider_info.name,
                        &provider_info.support_url,
                    )
                    .await?;

                info!(
                    code = %LogCode::Webhook,
                    provider = %provider,
                    bot_id = %bot_id,
                    "Test webhook processed"
                );

                Ok(Json(MessageResponse {
                    message: "Test webhook received".to_string(),
                }))
            } else {
                warn!(
                    code = %LogCode::Webhook,
                    provider = %provider,
                    bot_id = %bot_id,
                    "Bot owner not found for test webhook email"
                );

                Ok(Json(MessageResponse {
                    message: "Test webhook received, but bot owner not found for email".to_string(),
                }))
            }
        }
        ProviderResponse::Ignored => {
            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                "Webhook ignored after processing"
            );

            Ok(Json(MessageResponse {
                message: "Webhook ignored".to_string(),
            }))
        }
    }
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/webhooks/{provider}", post().to(vote_webhook));
}
