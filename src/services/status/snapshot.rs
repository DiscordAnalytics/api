use crate::domain::models::{ShardStatusSnapshot, StatusCheck, StatusCurrent};

pub(crate) fn create_status_snapshot(current: &StatusCurrent) -> StatusCheck {
    StatusCheck {
        bot_id: current.bot_id.clone(),
        deployment_mode: current.deployment_mode,
        timestamp: current.updated_at,
        status: current.status,
        total_shards: current.total_shards,
        shards: current
            .shards
            .iter()
            .map(|shard| ShardStatusSnapshot {
                shard_id: shard.shard_id,
                status: shard.status,
                latency_ms: shard.latency_ms,
                guild_count: shard.guild_count,
            })
            .collect(),
    }
}
