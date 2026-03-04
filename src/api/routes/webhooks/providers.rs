use std::str::from_utf8;

use actix_web::http::header::HeaderMap;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use ring::hmac::{HMAC_SHA256, Key, sign};
use serde_json::{Value, from_value};
use tracing::info;

use crate::{
    domain::{
        error::{ApiError, ApiResult},
        models::Bot,
    },
    openapi::schemas::{
        BotListMePayload, DBListPayload, DiscordListPayload, DiscordPlacePayload,
        DiscordsComPayload, TopGGPayload,
    },
    utils::logger::LogCode,
};

pub struct VoteResult {
    pub vote_count: i32,
    pub voter_id: String,
}

pub enum ProviderResponse {
    Vote(VoteResult),
    TestWebhook,
    Ignored,
}

struct TopGGSignature {
    timestamp: String,
    signature: String,
}

fn extract_topgg_signature(headers: &HeaderMap) -> Option<TopGGSignature> {
    let signature_str = headers.get("x-topgg-signature")?.to_str().ok()?;
    let (t_part, v1_part) = signature_str.split_once(',')?;
    let timestamp = t_part.trim().strip_prefix("t=")?;
    let signature = v1_part.trim().strip_prefix("v1=")?;

    Some(TopGGSignature {
        timestamp: timestamp.to_string(),
        signature: signature.to_string(),
    })
}

fn compute_topgg_signature(secret: &str, timestamp: &str, body: &[u8]) -> String {
    let mac = Key::new(HMAC_SHA256, secret.as_bytes());
    let data = [timestamp.as_bytes(), body].concat();
    let signature = sign(&mac, &data);
    hex::encode(signature.as_ref())
}

fn verify_topgg_signature(signature: &str, computed_signature: &str) -> bool {
    signature == computed_signature
}

pub async fn handle_provider(
    provider: &str,
    body: Value,
    body_bytes: &[u8],
    authorization: Option<&str>,
    bot: &Bot,
    headers: &HeaderMap,
) -> ApiResult<ProviderResponse> {
    match provider {
        "botlistme" => handle_botlistme(body, bot, authorization).await,
        "dblist" => handle_dblist(body, bot, authorization).await,
        "discordlist" => handle_discordlist(body_bytes, bot).await,
        "discordplace" => handle_discordplace(body, bot, authorization).await,
        "discordscom" => handle_discordscom(body, bot, authorization).await,
        "topgg" => handle_topgg(body, body_bytes, bot, headers).await,
        _ => {
            info!(
                code = %LogCode::Webhook,
                provider = %provider,
                bot_id = %bot.bot_id,
                "Received webhook from unsupported provider, ignoring"
            );
            Ok(ProviderResponse::Ignored)
        }
    }
}

async fn handle_botlistme(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("botlistme").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for botlist.me".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    if let Some(auth) = authorization
        && auth != webhook_secret
    {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = match from_value::<BotListMePayload>(body) {
        Ok(p) => p,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    if payload.bot != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.vote_type == "Test" {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: 1,
        voter_id: payload.user,
    }))
}

async fn handle_dblist(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("dblist").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for dblist".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    if let Some(auth) = authorization
        && auth != webhook_secret
    {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = match from_value::<DBListPayload>(body) {
        Ok(p) => p,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    if payload.bot_id != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.promotable_bot.is_some() {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: 1,
        voter_id: payload.id,
    }))
}

async fn handle_discordlist(body_bytes: &[u8], bot: &Bot) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("discordlist").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for discordlist".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    let token_data = match decode::<DiscordListPayload>(
        from_utf8(body_bytes).unwrap_or_default(),
        &DecodingKey::from_secret(webhook_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    let payload = token_data.claims;

    if payload.bot_id != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.is_test {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: 1,
        voter_id: payload.user_id,
    }))
}

async fn handle_discordplace(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("discordplace").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for discord.place".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    if let Some(auth) = authorization
        && auth != webhook_secret
    {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = match from_value::<DiscordPlacePayload>(body) {
        Ok(p) => p,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    if payload.bot != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.test {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: 1,
        voter_id: payload.user,
    }))
}

async fn handle_discordscom(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("discordscom").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for discords.com".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    if let Some(auth) = authorization
        && auth != webhook_secret
    {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = match from_value::<DiscordsComPayload>(body) {
        Ok(p) => p,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    if payload.bot != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.type_ == "review" {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.type_ == "test" {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: if payload.type_ == "premium_vote" {
            2
        } else {
            1
        },
        voter_id: payload.user,
    }))
}

async fn handle_topgg(
    body: Value,
    body_bytes: &[u8],
    bot: &Bot,
    headers: &HeaderMap,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.get("topgg").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for top.gg".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    let signature = match extract_topgg_signature(headers) {
        Some(sig) => sig,
        None => return Ok(ProviderResponse::Ignored),
    };

    let computed_signature =
        compute_topgg_signature(webhook_secret, &signature.timestamp, body_bytes);

    if !verify_topgg_signature(&signature.signature, &computed_signature) {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = match from_value::<TopGGPayload>(body) {
        Ok(p) => p,
        Err(_) => return Ok(ProviderResponse::Ignored),
    };

    let project = payload.data.project;

    if &project.type_ != "bot" {
        return Ok(ProviderResponse::Ignored);
    }

    if &project.platform != "discord" {
        return Ok(ProviderResponse::Ignored);
    }

    if project.platform_id != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    if payload.type_ == "webhook.test" {
        return Ok(ProviderResponse::TestWebhook);
    }

    Ok(ProviderResponse::Vote(VoteResult {
        vote_count: payload.data.weight.unwrap_or(1),
        voter_id: payload.data.user.platform_id,
    }))
}
