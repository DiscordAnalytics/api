use mongodb::{
    Collection, Database, IndexModel,
    bson::{DateTime, doc, serialize_to_bson},
    error::Result,
    options::IndexOptions,
};

use crate::{
    domain::models::{
        BotDeploymentMode, OverallStatus, ShardCurrentStatus, ShardStatus, StatusCurrent,
    },
    repository::common::ensure_collection,
    utils::constants::STATUS_CURRENT_COLLECTION,
};

#[derive(Clone)]
pub struct StatusCurrentRepository {
    collection: Collection<StatusCurrent>,
}

impl StatusCurrentRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        let collection = ensure_collection(db, STATUS_CURRENT_COLLECTION).await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "botId": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        Ok(Self { collection })
    }

    pub async fn find_by_bot_id(&self, bot_id: &str) -> Result<Option<StatusCurrent>> {
        self.collection.find_one(doc! { "botId": bot_id }).await
    }

    pub async fn create_status(
        &self,
        bot_id: &str,
        deployment_mode: BotDeploymentMode,
        total_shards: u32,
    ) -> Result<DateTime> {
        let now = DateTime::now();

        let shards = if deployment_mode == BotDeploymentMode::Sharded {
            (0..total_shards)
                .map(|shard_id| ShardCurrentStatus {
                    shard_id,
                    status: ShardStatus::Unknown,
                    latency_ms: None,
                    guild_count: None,
                    last_heartbeat_at: None,
                    updated_at: now,
                })
                .collect()
        } else {
            Vec::new()
        };

        self.collection
            .insert_one(StatusCurrent {
                bot_id: bot_id.to_string(),
                deployment_mode,
                status: OverallStatus::Unknown,
                total_shards,
                shards,
                latency_ms: None,
                guild_count: None,
                last_heartbeat_at: None,
                created_at: now,
                updated_at: now,
                last_evaluated_at: now,
            })
            .await?;

        Ok(now)
    }

    pub async fn update_status_config(
        &self,
        bot_id: &str,
        deployment_mode: BotDeploymentMode,
        total_shards: u32,
    ) -> Result<DateTime> {
        let now = DateTime::now();

        let shards = if deployment_mode == BotDeploymentMode::Sharded {
            (0..total_shards)
                .map(|shard_id| ShardCurrentStatus {
                    shard_id,
                    status: ShardStatus::Unknown,
                    latency_ms: None,
                    guild_count: None,
                    last_heartbeat_at: None,
                    updated_at: now,
                })
                .collect()
        } else {
            Vec::new()
        };

        self.collection
            .update_one(
                doc! { "botId": bot_id },
                doc! {
                    "$set": {
                        "deploymentMode": serialize_to_bson(&deployment_mode)?,
                        "totalShards": total_shards,
                        "status": serialize_to_bson(&OverallStatus::Unknown)?,
                        "shards": serialize_to_bson(&shards)?,
                        "latencyMs": None::<i64>,
                        "guildCount": None::<i64>,
                        "lastHeartbeatAt": None::<DateTime>,
                        "updatedAt": now,
                        "lastEvaluatedAt": now,
                    }
                },
            )
            .await?;

        Ok(now)
    }

    pub async fn update_shard_heartbeat(
        &self,
        bot_id: &str,
        shard_id: u32,
        latency_ms: Option<u32>,
        guild_count: Option<i64>,
    ) -> Result<Option<DateTime>> {
        let now = DateTime::now();

        let mut set = doc! {
          "shards.$.status": serialize_to_bson(&ShardStatus::Operational)?,
          "shards.$.lastHeartbeatAt": now,
          "shards.$.updatedAt": now,
          "updatedAt": now,
        };

        if let Some(latency_ms) = latency_ms {
            set.insert("shards.$.latencyMs", i64::from(latency_ms));
        }
        if let Some(guild_count) = guild_count {
            set.insert("shards.$.guildCount", guild_count);
        }

        let result = self
            .collection
            .update_one(
                doc! {
                  "botId": bot_id,
                  "shards.shardId": shard_id
                },
                doc! {
                    "$set": set,
                },
            )
            .await?;

        if result.matched_count == 0 {
            return Ok(None);
        }

        Ok(Some(now))
    }

    pub async fn update_single_heartbeat(
        &self,
        bot_id: &str,
        latency_ms: Option<u32>,
        guild_count: Option<i64>,
    ) -> Result<Option<DateTime>> {
        let now = DateTime::now();

        let result = self
            .collection
            .update_one(
                doc! { "botId": bot_id },
                doc! {
                    "$set": {
                        "status": serialize_to_bson(&OverallStatus::Operational)?,
                        "latencyMs": latency_ms.map(i64::from),
                        "guildCount": guild_count,
                        "lastHeartbeatAt": now,
                        "updatedAt": now,
                    }
                },
            )
            .await?;

        if result.matched_count == 0 {
            return Ok(None);
        }

        Ok(Some(now))
    }

    pub async fn update_overall_status(&self, bot_id: &str, status: OverallStatus) -> Result<()> {
        self.collection
            .update_one(
                doc! { "botId": bot_id },
                doc! {
                    "$set": {
                        "status": serialize_to_bson(&status)?,
                        "updatedAt": DateTime::now(),
                    }
                },
            )
            .await?;

        Ok(())
    }
}
