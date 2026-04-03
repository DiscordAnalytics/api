use std::{collections::HashMap, str::from_utf8};

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

pub struct ProviderInfo {
    pub name: String,
    pub support_url: String,
}

fn extract_discordlist_payload(secret: &str, body_bytes: &[u8]) -> Option<DiscordListPayload> {
    let body_str = from_utf8(body_bytes).ok()?;
    let token_data = decode::<DiscordListPayload>(
        body_str,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;
    Some(token_data.claims)
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
    let mut data = Vec::with_capacity(timestamp.len() + 1 + body.len());
    data.extend_from_slice(timestamp.as_bytes());
    data.push(b'.');
    data.extend_from_slice(body);
    let signature = sign(&mac, &data);
    hex::encode(signature.as_ref())
}

fn verify_topgg_signature(signature: &str, computed_signature: &str) -> bool {
    signature == computed_signature
}

pub fn get_provider_info(provider: &str) -> Option<ProviderInfo> {
    let infos = HashMap::from([
        ("botlistme", ("botlist.me", "https://discord.botlist.me")),
        ("dblist", ("discordbotlist", "support@discordbotlist.com")),
        (
            "discordlist",
            ("DiscordList", "https://discordlist.gg/help"),
        ),
        (
            "discordplace",
            ("discord.place", "https://invite.discord.place"),
        ),
        (
            "discordscom",
            (
                "discords.com",
                "https://docs.discords.com/discords.com-bots/receiving-webhooks",
            ),
        ),
        ("topgg", ("top.gg", "https://support.top.gg")),
        ("test", ("Test", "https://discordanalytics.xyz/support")),
    ]);

    infos.get(provider).map(|(name, support_url)| ProviderInfo {
        name: name.to_string(),
        support_url: support_url.to_string(),
    })
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
        "test" => Ok(ProviderResponse::TestWebhook),
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
    let webhook_config = bot
        .webhooks_config
        .webhooks
        .get("botlistme")
        .ok_or_else(|| {
            ApiError::WebhookError(
                "Bot does not have webhook configured for botlist.me".to_string(),
            )
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

    let payload = from_value::<BotListMePayload>(body)
        .map_err(|_| ApiError::InvalidInput("Invalid botlist.me payload".to_string()))?;

    if payload.bot != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    match payload.vote_type.as_str() {
        "Test" => Ok(ProviderResponse::TestWebhook),
        "Upvote" => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: 1,
            voter_id: payload.user,
        })),
        _ => Ok(ProviderResponse::Ignored),
    }
}

async fn handle_dblist(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.webhooks.get("dblist").ok_or_else(|| {
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

    let payload = from_value::<DBListPayload>(body)
        .map_err(|_| ApiError::InvalidInput("Invalid DBList payload".to_string()))?;

    if payload.bot_id != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    match payload.promotable_bot {
        Some(_) => Ok(ProviderResponse::TestWebhook),
        None => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: 1,
            voter_id: payload.id,
        })),
    }
}

async fn handle_discordlist(body_bytes: &[u8], bot: &Bot) -> ApiResult<ProviderResponse> {
    let webhook_config = bot
        .webhooks_config
        .webhooks
        .get("discordlist")
        .ok_or_else(|| {
            ApiError::WebhookError(
                "Bot does not have webhook configured for discordlist".to_string(),
            )
        })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    let payload = extract_discordlist_payload(webhook_secret, body_bytes).ok_or_else(|| {
        ApiError::InvalidInput("Invalid DiscordList payload or signature".to_string())
    })?;

    if payload.bot_id != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    match payload.is_test {
        true => Ok(ProviderResponse::TestWebhook),
        false => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: 1,
            voter_id: payload.user_id,
        })),
    }
}

async fn handle_discordplace(
    body: Value,
    bot: &Bot,
    authorization: Option<&str>,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot
        .webhooks_config
        .webhooks
        .get("discordplace")
        .ok_or_else(|| {
            ApiError::WebhookError(
                "Bot does not have webhook configured for discord.place".to_string(),
            )
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

    let payload = from_value::<DiscordPlacePayload>(body)
        .map_err(|_| ApiError::InvalidInput("Invalid discord.place payload".to_string()))?;

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
    let webhook_config = bot
        .webhooks_config
        .webhooks
        .get("discordscom")
        .ok_or_else(|| {
            ApiError::WebhookError(
                "Bot does not have webhook configured for discords.com".to_string(),
            )
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

    let payload = from_value::<DiscordsComPayload>(body)
        .map_err(|_| ApiError::InvalidInput("Invalid discords.com payload".to_string()))?;

    if payload.bot != bot.bot_id {
        return Ok(ProviderResponse::Ignored);
    }

    match payload.type_.as_str() {
        "test" => Ok(ProviderResponse::TestWebhook),
        "premium_vote" => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: 2,
            voter_id: payload.user,
        })),
        "vote" => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: 1,
            voter_id: payload.user,
        })),
        _ => Ok(ProviderResponse::Ignored),
    }
}

async fn handle_topgg(
    body: Value,
    body_bytes: &[u8],
    bot: &Bot,
    headers: &HeaderMap,
) -> ApiResult<ProviderResponse> {
    let webhook_config = bot.webhooks_config.webhooks.get("topgg").ok_or_else(|| {
        ApiError::WebhookError("Bot does not have webhook configured for top.gg".to_string())
    })?;

    let webhook_secret = match &webhook_config.webhook_secret {
        Some(secret) if !secret.is_empty() => secret,
        _ => return Ok(ProviderResponse::Ignored),
    };

    let signature = extract_topgg_signature(headers).ok_or_else(|| {
        ApiError::WebhookError("Missing or invalid TopGG signature header".to_string())
    })?;
    let computed_signature =
        compute_topgg_signature(webhook_secret, &signature.timestamp, body_bytes);

    if !verify_topgg_signature(&signature.signature, &computed_signature) {
        return Ok(ProviderResponse::Ignored);
    }

    let payload = from_value::<TopGGPayload>(body)
        .map_err(|_| ApiError::InvalidInput("Invalid TopGG payload".to_string()))?;

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

    match payload.type_.as_str() {
        "vote.create" => Ok(ProviderResponse::Vote(VoteResult {
            vote_count: payload.data.weight.unwrap_or(1),
            voter_id: payload.data.user.platform_id,
        })),
        "webhook.test" => Ok(ProviderResponse::TestWebhook),
        _ => Ok(ProviderResponse::Ignored),
    }
}
