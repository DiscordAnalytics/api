use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};
use chrono::Utc;

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::{BotDeploymentMode, ShardStatus, ShardStatusSnapshot},
    },
    openapi::schemas::{HeartbeatRequest, HeartbeatResponse},
    repository::{Repositories, status_pending_bucket::StatusPendingBucketRepository},
    utils::{logger::LogCode, status::MAX_TOTAL_SHARDS},
};

#[api_operation(
    summary = "Receive an SDK heartbeat",
    description = "Receive a heartbeat from the SDK to keep the connection alive.",
    tag = "Status"
)]
async fn update_heartbeat(
    auth: Authenticated,
    repos: Data<Repositories>,
    Json(payload): Json<HeartbeatRequest>,
) -> ApiResult<Json<HeartbeatResponse>> {
    info!(
        code = %LogCode::Status,
        "Receiving heartbeat from SDK"
    );

    let ctx = &auth;

    if !ctx.is_bot() {
        return Err(ApiError::Forbidden);
    }

    let Some(token) = &ctx.token else {
        return Err(ApiError::Unauthorized);
    };

    let bot = repos.bots.find_by_token(token).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Status,
            "Invalid bot token for heartbeat"
        );
        ApiError::Unauthorized
    })?;

    if bot.suspended {
        return Err(ApiError::BotSuspended);
    }

    let current_status = repos
        .status_current
        .find_by_bot_id(&bot.bot_id)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Status,
                bot_id = %bot.bot_id,
                "No status configuration found for bot heartbeat"
            );
            ApiError::NotFound("No status configuration found".into())
        })?;

    let shard_id = match current_status.deployment_mode {
        BotDeploymentMode::Single => {
            if payload.shard_id.is_some() || payload.total_shards.is_some_and(|count| count != 1) {
                return Err(ApiError::BadRequest(
                    "Invalid shard configuration for single deployment".into(),
                ));
            }
            0
        }
        BotDeploymentMode::Sharded => {
            let (Some(shard_id), Some(total_shards)) = (payload.shard_id, payload.total_shards)
            else {
                return Err(ApiError::BadRequest(
                    "Sharded deployments require shardId and totalShards".into(),
                ));
            };

            if total_shards == 0 {
                return Err(ApiError::BadRequest(
                    "totalShards must be greater than zero".into(),
                ));
            }

            if total_shards > MAX_TOTAL_SHARDS {
                return Err(ApiError::BadRequest(format!(
                    "totalShards must be less than or equal to {MAX_TOTAL_SHARDS}"
                )));
            }

            if total_shards != current_status.total_shards {
                return Err(ApiError::Conflict(
                    "Heartbeat totalShards does not match configured totalShards".into(),
                ));
            }

            if shard_id >= total_shards {
                return Err(ApiError::BadRequest(
                    "shardId must be lower than totalShards".into(),
                ));
            }

            shard_id
        }
    };

    // Update the live status (latency, guild_count, shard status).
    let updated_status = repos
        .status_current
        .update_shard_heartbeat(
            &bot.bot_id,
            current_status.deployment_mode,
            current_status.total_shards,
            shard_id,
            payload.latency_ms,
            payload.guild_count,
        )
        .await?
        .ok_or(ApiError::Conflict(
            "Bot status configuration changed while processing heartbeat. Please retry".into(),
        ))?;

    let received_at = updated_status
        .last_heartbeat_at
        .ok_or(ApiError::NotFound("Heartbeat timestamp not found".into()))?;

    // Write into the pending bucket for the current 5-min window.
    // The sealing task will finalize it into StatusChecks + StatusChecksHourly.
    let now_ms = Utc::now().timestamp_millis();
    let bucket_ts = StatusPendingBucketRepository::bucket_timestamp(now_ms);

    let shard_snapshot = ShardStatusSnapshot {
        shard_id,
        status: updated_status
            .shards
            .iter()
            .find(|s| s.shard_id == shard_id)
            .map(|s| s.status)
            .unwrap_or(ShardStatus::Unknown),
        latency_ms: payload.latency_ms,
        guild_count: payload.guild_count,
    };

    repos
        .status_pending_bucket
        .upsert_shard(
            &bot.bot_id,
            updated_status,
            bucket_ts,
            shard_snapshot,
            received_at,
        )
        .await?;

    info!(
        code = %LogCode::Status,
        bot_id = %bot.bot_id,
        "Heartbeat processed successfully"
    );

    Ok(Json(HeartbeatResponse {
        success: true,
        message: "Heartbeat received successfully".into(),
        received_at: received_at.to_chrono(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/heartbeat", post().to(update_heartbeat));
}
