use std::time::Duration;

use futures::StreamExt;
use mongodb::{
    Collection, Database, IndexModel,
    bson::{DateTime, doc, serialize_to_bson},
    error::Result,
    options::{FindOneAndUpdateOptions, IndexOptions, ReturnDocument},
};

use crate::{
    domain::models::{ShardStatusSnapshot, StatusCurrent, StatusPendingBucket},
    utils::constants::{
        MAX_SEAL_BATCH_SIZE, STATUS_BUCKET_WINDOW_SECS, STATUS_PENDING_BUCKET_TTL_SECS,
        STATUS_PENDING_BUCKETS_COLLECTION,
    },
};

use super::{common::ensure_collection, status_check_hourly::StatusCheckHourlyRepository};

#[derive(Clone)]
pub struct StatusPendingBucketRepository {
    collection: Collection<StatusPendingBucket>,
}

impl StatusPendingBucketRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        let collection = ensure_collection(db, STATUS_PENDING_BUCKETS_COLLECTION).await?;

        // Lookup index for upsert / sealing queries.
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "botId": 1, "bucketTimestamp": -1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
            )
            .await?;

        // Index for sealing task: find unsealed buckets older than the window.
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "sealed": 1, "bucketTimestamp": 1 })
                    .build(),
            )
            .await?;

        // Safety-net TTL: auto-remove even if sealing fails.
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "createdAt": 1 })
                    .options(
                        IndexOptions::builder()
                            .expire_after(Duration::from_secs(STATUS_PENDING_BUCKET_TTL_SECS))
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(Self { collection })
    }

    /// Floor `now_ms` to the nearest bucket boundary.
    pub fn bucket_timestamp(now_ms: i64) -> DateTime {
        let window_ms = STATUS_BUCKET_WINDOW_SECS * 1000;
        DateTime::from_millis((now_ms / window_ms) * window_ms)
    }

    /// Upsert the bucket for this bot / window, then atomically merge the
    /// incoming shard slot. Returns the updated document so the caller can
    /// derive the overall status.
    pub async fn upsert_shard(
        &self,
        bot_id: &str,
        updated_status: StatusCurrent,
        bucket_ts: DateTime,
        shard: ShardStatusSnapshot,
        now: DateTime,
    ) -> Result<Option<StatusPendingBucket>> {
        let deployment_bson = serialize_to_bson(&updated_status.deployment_mode)?;
        let overall_bson = serialize_to_bson(&updated_status.status)?;
        let shard_status_bson = serialize_to_bson(&shard.status)?;

        let shard_doc = doc! {
            "shardId": shard.shard_id,
            "status": shard_status_bson.clone(),
            "latencyMs": shard.latency_ms.map(|v| v as i64),
            "guildCount": shard.guild_count,
        };

        // If the shard slot already exists in the array, update it in-place.
        // Otherwise push a new slot. We use an aggregation pipeline update so
        // we can use $map / $concatArrays.
        let updated_shards = doc! {
            "$let": {
                "vars": { "incoming": shard_doc.clone() },
                "in": {
                    "$let": {
                        "vars": {
                            "exists": {
                                "$anyElementTrue": [{
                                    "$map": {
                                        "input": "$shards",
                                        "as": "s",
                                        "in": { "$eq": ["$$s.shardId", shard.shard_id as i64] }
                                    }
                                }]
                            }
                        },
                        "in": {
                            "$cond": [
                                "$$exists",
                                // Replace the matching slot.
                                {
                                    "$map": {
                                        "input": "$shards",
                                        "as": "s",
                                        "in": {
                                            "$cond": [
                                                { "$eq": ["$$s.shardId", shard.shard_id as i64] },
                                                { "$mergeObjects": ["$$s", "$$incoming"] },
                                                "$$s"
                                            ]
                                        }
                                    }
                                },
                                // Append a new slot.
                                { "$concatArrays": ["$shards", ["$$incoming"]] }
                            ]
                        }
                    }
                }
            }
        };

        // Recompute overall status from the merged shards array using the same
        // logic as StatusCurrentRepository. We re-use the worst-case evaluation.
        let new_overall = StatusCheckHourlyRepository::worst_status_expr(&overall_bson);

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .clone_with_type::<StatusPendingBucket>()
            .find_one_and_update(
                doc! {
                    "botId": bot_id,
                    "bucketTimestamp": bucket_ts,
                    "sealed": false,
                },
                vec![doc! {
                    "$set": {
                        "botId": bot_id,
                        "deploymentMode": deployment_bson,
                        "bucketTimestamp": bucket_ts,
                        "totalShards": updated_status.total_shards as i64,
                        "shards": updated_shards,
                        "status": new_overall,
                        "sealed": false,
                        "createdAt": { "$ifNull": ["$createdAt", now] },
                        "updatedAt": now,
                    }
                }],
            )
            .with_options(options)
            .await
    }

    /// Find all unsealed buckets whose window has fully elapsed.
    pub async fn find_expired_unsealed(
        &self,
        cutoff: DateTime,
    ) -> Result<Vec<StatusPendingBucket>> {
        let mut cursor = self
            .collection
            .find(doc! {
                "sealed": false,
                "bucketTimestamp": { "$lte": cutoff }
            })
            .limit(MAX_SEAL_BATCH_SIZE)
            .await?;

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            results.push(doc?);
        }
        Ok(results)
    }

    /// Mark a bucket as sealed so it won't be processed again.
    pub async fn mark_sealed(&self, bot_id: &str, bucket_ts: DateTime) -> Result<()> {
        self.collection
            .update_one(
                doc! { "botId": bot_id, "bucketTimestamp": bucket_ts },
                doc! { "$set": { "sealed": true } },
            )
            .await?;
        Ok(())
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<()> {
        self.collection
            .delete_many(doc! { "botId": bot_id })
            .await?;
        Ok(())
    }
}
