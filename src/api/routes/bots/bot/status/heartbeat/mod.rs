use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, post},
};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::{
            BotDeploymentMode, OverallStatus, ShardCurrentStatus, ShardStatus, ShardStatusSnapshot,
            StatusCheck,
        },
    },
    openapi::schemas::{HeartbeatRequest, HeartbeatResponse},
    repository::Repositories,
    utils::logger::LogCode,
};

fn calculate_overall_status(shards: &[ShardCurrentStatus]) -> OverallStatus {
    if shards.is_empty() {
        return OverallStatus::Unknown;
    }

    let mut has_offline = false;
    let mut has_degraded = false;
    let mut all_offline = true;
    let mut all_unknown = true;
    let mut all_operational = true;

    for shard in shards {
        match shard.status {
            ShardStatus::Offline => {
                has_offline = true;
                all_unknown = false;
                all_operational = false;
            }
            ShardStatus::Degraded => {
                has_degraded = true;
                all_offline = false;
                all_unknown = false;
                all_operational = false;
            }
            ShardStatus::Operational => {
                all_offline = false;
                all_unknown = false;
            }
            ShardStatus::Unknown => {
                all_offline = false;
                all_operational = false;
            }
        }
    }

    if all_unknown {
        OverallStatus::Unknown
    } else if all_offline {
        OverallStatus::MajorOutage
    } else if has_offline {
        OverallStatus::PartialOutage
    } else if has_degraded {
        OverallStatus::Degraded
    } else if all_operational {
        OverallStatus::Operational
    } else {
        OverallStatus::Unknown
    }
}

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

    match current_status.deployment_mode {
        BotDeploymentMode::Single => {
            if payload.shard_id.is_some() || payload.total_shards.is_some_and(|count| count != 1) {
                return Err(ApiError::BadRequest(
                    "Invalid shard configuration for single deployment".into(),
                ));
            }
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
        }
    }

    let received_at = match current_status.deployment_mode {
        BotDeploymentMode::Single => {
            repos
                .status_current
                .update_single_heartbeat(&bot.bot_id, payload.latency_ms, payload.guild_count)
                .await?
        }
        BotDeploymentMode::Sharded => {
            repos
                .status_current
                .update_shard_heartbeat(
                    &bot.bot_id,
                    payload.shard_id.unwrap(),
                    payload.latency_ms,
                    payload.guild_count,
                )
                .await?
        }
    }
    .ok_or(ApiError::NotFound(
        "Status configuration or shard not found".into(),
    ))?;

    let updated_status = repos
        .status_current
        .find_by_bot_id(&bot.bot_id)
        .await?
        .ok_or(ApiError::NotFound("Status configuration not found".into()))?;

    let overall_status = match updated_status.deployment_mode {
        BotDeploymentMode::Single => OverallStatus::Operational,
        BotDeploymentMode::Sharded => calculate_overall_status(&updated_status.shards),
    };

    repos
        .status_current
        .update_overall_status(&bot.bot_id, overall_status)
        .await?;

    let check = StatusCheck {
        bot_id: bot.bot_id.clone(),
        deployment_mode: updated_status.deployment_mode,
        timestamp: received_at,
        status: overall_status,
        total_shards: updated_status.total_shards,
        shards: updated_status
            .shards
            .iter()
            .map(|shard| ShardStatusSnapshot {
                shard_id: shard.shard_id,
                status: shard.status,
                latency_ms: shard.latency_ms,
                guild_count: shard.guild_count,
            })
            .collect(),
        latency_ms: updated_status.latency_ms,
        guild_count: updated_status.guild_count,
    };

    repos.status_check.create_check(check).await?;

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
