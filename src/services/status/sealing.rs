use chrono::Utc;
use mongodb::bson::DateTime;

use crate::{
    domain::{
        error::ApiResult,
        models::{ShardStatusSnapshot, StatusCheck},
    },
    repository::{Repositories, status_check_hourly::StatusCheckHourlyRepository},
    utils::constants::STATUS_BUCKET_WINDOW_SECS,
};

/// Seal all expired pending buckets:
/// 1. Find unsealed buckets whose window has fully elapsed.
/// 2. For each: insert a raw StatusCheck + upsert the hourly bucket.
/// 3. Mark the pending bucket as sealed.
pub async fn seal_expired_buckets(repos: &Repositories) -> ApiResult<()> {
    let now_ms = Utc::now().timestamp_millis();
    let cutoff = DateTime::from_millis(now_ms - STATUS_BUCKET_WINDOW_SECS * 1000);

    let expired = repos
        .status_pending_bucket
        .find_expired_unsealed(cutoff)
        .await?;

    for bucket in expired {
        let shards: Vec<ShardStatusSnapshot> = bucket
            .shards
            .iter()
            .map(|s| ShardStatusSnapshot {
                shard_id: s.shard_id,
                status: s.status,
                latency_ms: s.latency_ms,
                guild_count: s.guild_count,
            })
            .collect();

        let now = DateTime::now();

        // 1. Insert raw StatusCheck (7-day TTL).
        let check = StatusCheck {
            bot_id: bucket.bot_id.clone(),
            deployment_mode: bucket.deployment_mode,
            timestamp: bucket.bucket_timestamp,
            status: bucket.status,
            total_shards: bucket.total_shards,
            shards: shards.clone(),
        };
        repos.status_check.create_check(check).await?;

        // 2. Upsert hourly bucket (90-day TTL).
        let hour_ts =
            StatusCheckHourlyRepository::hour_timestamp(bucket.bucket_timestamp.timestamp_millis());
        repos
            .status_check_hourly
            .upsert_from_check(&bucket.bot_id, bucket.clone().into(), &shards, hour_ts, now)
            .await?;

        // 3. Mark sealed.
        repos
            .status_pending_bucket
            .mark_sealed(&bucket.bot_id, bucket.bucket_timestamp)
            .await?;
    }

    Ok(())
}
