use mongodb::{
    ClientSession, Collection, Database, IndexModel,
    bson::{Bson, DateTime, doc, serialize_to_bson},
    error::Result,
    options::{IndexOptions, ReturnDocument},
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
        let shards = Self::initialize_shards(total_shards, now);

        self.collection
            .insert_one(StatusCurrent {
                bot_id: bot_id.to_string(),
                deployment_mode,
                status: OverallStatus::Unknown,
                total_shards,
                shards,
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
    ) -> Result<Option<DateTime>> {
        let now = DateTime::now();
        let shards = Self::initialize_shards(total_shards, now);

        let result = self
            .collection
            .update_one(
                doc! {
                    "botId": bot_id,
                    "$or": [
                        {
                            "deploymentMode": {
                                "$ne": serialize_to_bson(&deployment_mode)?
                            }
                        },
                        {
                            "totalShards": {
                                "$ne": total_shards
                            }
                        }
                    ]
                },
                doc! {
                    "$set": {
                        "deploymentMode": serialize_to_bson(&deployment_mode)?,
                        "totalShards": total_shards,
                        "status": serialize_to_bson(&OverallStatus::Unknown)?,
                        "shards": serialize_to_bson(&shards)?,
                        "lastHeartbeatAt": None::<DateTime>,
                        "updatedAt": now,
                        "lastEvaluatedAt": now,
                    }
                },
            )
            .await?;

        if result.matched_count > 0 {
            return Ok(Some(now));
        }

        let exists = self
            .collection
            .find_one(doc! { "botId": bot_id })
            .await?
            .is_some();

        Ok(exists.then_some(now))
    }

    pub async fn update_shard_heartbeat(
        &self,
        bot_id: &str,
        shard_id: u32,
        latency_ms: Option<u32>,
        guild_count: Option<i64>,
    ) -> Result<Option<StatusCurrent>> {
        let now = DateTime::now();

        let mut shard_update = doc! {
            "status": serialize_to_bson(&ShardStatus::Operational)?,
            "lastHeartbeatAt": now,
            "updatedAt": now,
        };

        if let Some(latency_ms) = latency_ms {
            shard_update.insert("latencyMs", i64::from(latency_ms));
        }
        if let Some(guild_count) = guild_count {
            shard_update.insert("guildCount", guild_count);
        }

        let updated_shards = doc! {
            "$map": {
                "input": "$shards",
                "as": "shard",
                "in": {
                    "$cond": [
                        {
                            "$eq": [
                                "$$shard.shardId",
                                i64::from(shard_id),
                            ]
                        },
                        {
                            "$mergeObjects": [
                                "$$shard",
                                shard_update
                            ]
                        },
                        "$$shard"
                    ]
                }
            }
        };

        let update_shard_stage = doc! {
            "$set": {
                "shards": updated_shards,
                "lastHeartbeatAt": now,
                "updatedAt": now,
            }
        };

        let overall_status = Self::overall_status_expression()?;

        let update_status_stage = doc! {
            "$set": {
                "status": overall_status,
                "lastEvaluatedAt": now,
            }
        };

        let result = self
            .collection
            .find_one_and_update(
                doc! {
                  "botId": bot_id,
                  "shards.shardId": shard_id
                },
                vec![update_shard_stage, update_status_stage],
            )
            .return_document(ReturnDocument::After)
            .await?;

        Ok(result)
    }

    pub async fn find_bots_with_stale_heartbeats(&self, cutoff: DateTime) -> Result<Vec<String>> {
        let offline = serialize_to_bson(&ShardStatus::Offline)?;

        let stale_shard_filter = doc! {
            "lastHeartbeatAt": {
                "$lte": cutoff,
                "$ne": Bson::Null,
            },
            "status": {
                "$ne": offline.clone(),
            },
        };

        let bot_ids = self
            .collection
            .distinct(
                "botId",
                doc! {
                    "shards": {
                        "$elemMatch": stale_shard_filter,
                    }
                },
            )
            .await?;

        Ok(bot_ids
            .into_iter()
            .filter_map(|id| match id {
                Bson::String(id) => Some(id),
                _ => None,
            })
            .collect())
    }

    pub async fn expire_stale_heartbeats(
        &self,
        bot_id: &str,
        cutoff: DateTime,
        session: &mut ClientSession,
    ) -> Result<Option<StatusCurrent>> {
        let now = DateTime::now();
        let offline = serialize_to_bson(&ShardStatus::Offline)?;

        let stale_shard_filter = doc! {
            "lastHeartbeatAt": {
                "$lte": cutoff,
                "$ne": Bson::Null,
            },
            "status": {
                "$ne": offline.clone(),
            },
        };

        let updated_shards = doc! {
            "$map": {
                "input": "$shards",
                "as": "shard",
                "in": {
                    "$cond": [
                        {
                            "$and": [
                                {
                                    "$ne": [
                                        "$$shard.lastHeartbeatAt",
                                        Bson::Null,
                                    ]
                                },
                                {
                                    "$lte": [
                                        "$$shard.lastHeartbeatAt",
                                        cutoff,
                                    ]
                                },
                                {
                                    "$ne": [
                                        "$$shard.status",
                                        offline.clone(),
                                    ]
                                }
                            ]
                        },
                        {
                            "$mergeObjects": [
                                "$$shard",
                                {
                                    "status": offline.clone(),
                                    "updatedAt": now,
                                }
                            ]
                        },
                        "$$shard"
                    ]
                }
            }
        };

        let result = self
            .collection
            .find_one_and_update(
                doc! {
                    "botId": bot_id,
                    "shards": {
                        "$elemMatch": stale_shard_filter,
                    }
                },
                vec![
                    doc! {
                        "$set": {
                            "shards": updated_shards,
                        }
                    },
                    doc! {
                        "$set": {
                            "status": Self::overall_status_expression()?,
                            "updatedAt": now,
                            "lastEvaluatedAt": now,
                        }
                    },
                ],
            )
            .session(&mut *session)
            .return_document(ReturnDocument::After)
            .await?;

        Ok(result)
    }

    fn initialize_shards(total_shards: u32, now: DateTime) -> Vec<ShardCurrentStatus> {
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
    }

    fn all_shards_match_status(status: Bson) -> Bson {
        doc! {
            "$allElementsTrue": [
                {
                    "$map": {
                        "input": "$$shards",
                        "as": "shard",
                        "in": {
                            "$eq": [
                                "$$shard.status",
                                status
                            ]
                        }
                    }
                }
            ]
        }
        .into()
    }

    fn any_shard_matches_status(status: Bson) -> Bson {
        doc! {
            "$anyElementTrue": [
                {
                    "$map": {
                        "input": "$$shards",
                        "as": "shard",
                        "in": {
                            "$eq": [
                                "$$shard.status",
                                status,
                            ]
                        }
                    }
                }
            ]
        }
        .into()
    }

    fn overall_status_expression() -> Result<Bson> {
        let unknown = serialize_to_bson(&OverallStatus::Unknown)?;
        let operational = serialize_to_bson(&ShardStatus::Operational)?;
        let degraded = serialize_to_bson(&ShardStatus::Degraded)?;
        let offline = serialize_to_bson(&ShardStatus::Offline)?;

        Ok(doc! {
            "$let": {
                "vars": {
                    "shards": "$shards",
                },
                "in": {
                    "$switch": {
                        "branches": [
                            {
                                "case": {
                                    "$and": [
                                        { "$gt": [{ "$size": "$$shards" }, 0] },
                                        Self::all_shards_match_status(unknown.clone())
                                    ]
                                },
                                "then": unknown.clone(),
                            },
                            {
                                "case": Self::all_shards_match_status(offline.clone()),
                                "then": serialize_to_bson(&OverallStatus::MajorOutage)?,
                            },
                            {
                                "case": Self::any_shard_matches_status(offline),
                                "then": serialize_to_bson(&OverallStatus::PartialOutage)?,
                            },
                            {
                                "case": Self::any_shard_matches_status(degraded),
                                "then": serialize_to_bson(&OverallStatus::Degraded)?,
                            },
                            {
                                "case": Self::all_shards_match_status(operational),
                                "then": serialize_to_bson(&OverallStatus::Operational)?,
                            },
                        ],
                        "default": unknown
                    }
                }
            }
        }
        .into())
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<()> {
        self.collection
            .delete_many(doc! { "botId": bot_id })
            .await?;
        Ok(())
    }
}
