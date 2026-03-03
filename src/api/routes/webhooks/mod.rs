mod providers;

use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use apistos::{
    api_operation,
    web::{ServiceConfig, post, resource, scope},
};
use serde_json::{Value, from_slice};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    api::{
        middleware::RawBody,
        routes::webhooks::providers::{ProviderResponse, handle_provider},
    },
    domain::error::{ApiError, ApiResult},
    managers::VotesWebhooksManager,
    openapi::schemas::VoteWebhookResponse,
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

fn extract_bot_id_from_payload(provider: &str, body: &Value) -> Option<String> {
    match provider {
        "topgg" => body
            .get("data")?
            .get("project")?
            .get("plaform_id")?
            .as_str()
            .map(String::from),
        "dblist" | "discordlist" => body.get("bot_id")?.as_str().map(String::from),
        "discordscom" | "botlistme" => body.get("bot")?.as_str().map(String::from),
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
) -> ApiResult<Json<VoteWebhookResponse>> {
    let provider = path.into_inner();

    let body_bytes = &body.0;

    let body_value: Value = from_slice(body_bytes).map_err(|e| {
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
            services
                .webhooks
                .record_vote(
                    &bot_id,
                    &vote_result.voter_id,
                    &provider,
                    vote_result.vote_count,
                )
                .await?;

            if provider != "test" {
                let _ = services
                    .webhooks
                    .trigger_webhook_notification(
                        &bot,
                        &vote_result.voter_id,
                        &provider,
                        body_value,
                        &webhook_manager,
                    )
                    .await;
            }

            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                voter_id = %vote_result.voter_id,
                vote_count = vote_result.vote_count,
                "Processed vote webhook successfully"
            );

            Ok(Json(VoteWebhookResponse {
                success: true,
                message: "Vote processed successfully".to_string(),
            }))
        }
        ProviderResponse::TestWebhook => {
            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                "Received test webhook, ignoring vote processing"
            );

            return Ok(Json(VoteWebhookResponse {
                success: true,
                message: "Test webhook received".to_string(),
            }));
        }
        ProviderResponse::Ignored => {
            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot_id,
                "Webhook ignored after processing"
            );

            Ok(Json(VoteWebhookResponse {
                success: true,
                message: "Webhook ignored".to_string(),
            }))
        }
    }
}

#[api_operation(
    summary = "Legacy webhook endpoint",
    description = "This endpoint is a legacy webhook handler that is now deprecated. It was previously used to receive webhooks from vote providers, but it has been replaced by the new /webhooks/{provider} endpoint. This endpoint will return a message indicating that it is deprecated and should not be used for new integrations.",
    tag = "Webhooks",
    deprecated
)]
async fn legacy_vote_webhook(
    req: HttpRequest,
    services: Data<Services>,
    repos: Data<Repositories>,
    webhook_manager: Data<Arc<Mutex<VotesWebhooksManager>>>,
    path: Path<(String, String)>,
    body: RawBody,
) -> ApiResult<HttpResponse> {
    let (bot_id, provider) = path.into_inner();

    warn!(
        code = %LogCode::Webhook,
        provider = %provider,
        bot_id = %bot_id,
        "Received webhook on legacy endpoint, this endpoint is deprecated and should not be used"
    );

    let mut body_with_bot_id: Value = from_slice(&body.0).unwrap_or(Value::Null);
    if let Value::Object(ref mut map) = body_with_bot_id {
        map.insert("bot_id".to_string(), Value::String(bot_id.clone()));
    }

    let result = vote_webhook(
        req,
        services,
        repos,
        webhook_manager,
        Path::from(provider.clone()),
        RawBody(body_with_bot_id.to_string().into_bytes()),
    )
    .await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            "X-Deprecation-Warning",
            "This endpoint is deprecated, please use POST /webhooks/{provider} instead",
        ))
        .insert_header(("X-New-Endpoint", format!("/webhooks/{}", provider)))
        .json(result.into_inner()))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(scope("/webhooks").service(resource("/{provider}").route(post().to(vote_webhook))));
}
