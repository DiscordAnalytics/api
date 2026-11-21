use std::{collections::HashMap, time::Duration};

use futures::StreamExt;
use mongodb::{
    Collection, Database, IndexModel,
    bson::{Bson, DateTime, Document, doc, serialize_to_bson},
    error::Result,
    options::{FindOneAndUpdateOptions, IndexOptions, ReturnDocument},
};

use crate::{
    domain::models::{
        OverallStatus, ShardHourlySnapshot, ShardStatus, ShardStatusSnapshot, StatusCheck,
        StatusCheckHourly, StoredStatusCheckHourly,
    },
    openapi::schemas::{HourlyHistoryCursor, HourlyHistoryPage},
    repository::common::ensure_collection,
    utils::constants::{STATUS_CHECK_HOURLY_TTL_SECS, STATUS_CHECKS_HOURLY_COLLECTION},
};

#[derive(Clone)]
pub struct StatusCheckHourlyRepository {
    collection: Collection<StatusCheckHourly>,
}

impl StatusCheckHourlyRepository {
    pub async fn new(db: &Database) -> Result<Self> {
        let collection = ensure_collection(db, STATUS_CHECKS_HOURLY_COLLECTION).await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "botId": 1, "hourTimestamp": -1, "_id": -1 })
                    .build(),
            )
            .await?;

        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "hourTimestamp": 1 })
                    .options(
                        IndexOptions::builder()
                            .expire_after(Duration::from_secs(STATUS_CHECK_HOURLY_TTL_SECS))
                            .build(),
                    )
                    .build(),
            )
            .await?;

        Ok(Self { collection })
    }

    /// Floor a millisecond timestamp to the hour boundary.
    pub fn hour_timestamp(ts_ms: i64) -> DateTime {
        let hour_ms: i64 = 60 * 60 * 1000;
        DateTime::from_millis((ts_ms / hour_ms) * hour_ms)
    }

    /// Upsert hourly bucket: worst-case overall status, running latency average,
    /// last-known guild count, incrementing sample_count.
    pub async fn upsert_from_check(
        &self,
        bot_id: &str,
        check: StatusCheck,
        shards: &[ShardStatusSnapshot],
        bucket_ts: DateTime,
        now: DateTime,
    ) -> Result<()> {
        let filter = doc! {
            "botId": bot_id,
            "hourTimestamp": bucket_ts,
        };

        // Fetch the existing hourly bucket.
        let existing = self.collection.find_one(filter.clone()).await?;

        let (existing_shards, existing_status) = match existing {
            Some(hourly) => (hourly.shards, Some(hourly.status)),
            None => (Vec::new(), None),
        };

        let merged_shards = Self::merge_shards(existing_shards, shards);

        // Preserve the worst overall status observed during the hour.
        let overall_status = existing_status
            .as_ref()
            .map(|existing| Self::worst_overall_status(existing, &check.status))
            .unwrap_or_else(|| check.status);

        let deployment_bson = serialize_to_bson(&check.deployment_mode)?;
        let status_bson = serialize_to_bson(&overall_status)?;
        let shards_bson = serialize_to_bson(&merged_shards)?;

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();

        self.collection
            .find_one_and_update(
                filter,
                doc! {
                    "$set": {
                        "deploymentMode": deployment_bson,
                        "totalShards": check.total_shards as i64,
                        "status": status_bson,
                        "shards": shards_bson,
                        "updatedAt": now,
                    },
                    "$setOnInsert": {
                        "botId": bot_id,
                        "hourTimestamp": bucket_ts,
                        "createdAt": now,
                    },
                },
            )
            .with_options(options)
            .await?;

        Ok(())
    }

    /// Paginated cursor-based query over hourly history.
    pub async fn find_history(
        &self,
        bot_id: &str,
        from: DateTime,
        to: DateTime,
        limit: u32,
        cursor: Option<HourlyHistoryCursor>,
    ) -> Result<HourlyHistoryPage> {
        let mut filter = doc! {
            "botId": bot_id,
            "hourTimestamp": { "$gte": from, "$lt": to },
        };

        if let Some(c) = cursor {
            filter.insert(
                "$or",
                vec![
                    doc! { "hourTimestamp": { "$lt": c.timestamp } },
                    doc! { "hourTimestamp": c.timestamp, "_id": { "$lt": c.id } },
                ],
            );
        }

        let collection = self.collection.clone_with_type::<StoredStatusCheckHourly>();

        let mut cursor = collection
            .find(filter)
            .sort(doc! { "hourTimestamp": -1i32, "_id": -1i32 })
            .limit(i64::from(limit) + 1)
            .await?;

        let mut checks = Vec::with_capacity(limit as usize + 1);
        while let Some(result) = cursor.next().await {
            checks.push(result?);
        }

        let has_more = checks.len() > limit as usize;
        if has_more {
            checks.pop();
        }

        Ok(HourlyHistoryPage { checks, has_more })
    }

    pub async fn delete_by_bot_id(&self, bot_id: &str) -> Result<()> {
        self.collection
            .delete_many(doc! { "botId": bot_id })
            .await?;
        Ok(())
    }

    pub(crate) fn worst_status_expr(incoming: &Bson) -> Document {
        let major = serialize_to_bson(&OverallStatus::MajorOutage).unwrap_or_default();
        let partial = serialize_to_bson(&OverallStatus::PartialOutage).unwrap_or_default();
        let degraded = serialize_to_bson(&OverallStatus::Degraded).unwrap_or_default();
        let operational = serialize_to_bson(&OverallStatus::Operational).unwrap_or_default();

        let to_severity = |status: Bson| {
            doc! {
                "$switch": {
                    "branches": [
                        { "case": { "$eq": [status.clone(), major.clone()] }, "then": 4i64 },
                        { "case": { "$eq": [status.clone(), partial.clone()] }, "then": 3i64 },
                        { "case": { "$eq": [status.clone(), degraded.clone()] }, "then": 2i64 },
                        { "case": { "$eq": [status, operational.clone()] }, "then": 1i64 },
                    ],
                    "default": 0i64,
                }
            }
        };

        // severity(incoming) >= severity(existing) -> take incoming.
        doc! {
            "$cond": [
                {
                    "$gte": [
                        to_severity(incoming.clone()),
                        to_severity(Bson::String("$status".to_owned())),
                    ]
                },
                incoming.clone(),
                "$status",
            ]
        }
    }

    fn merge_shards(
        existing: Vec<ShardHourlySnapshot>,
        incoming: &[ShardStatusSnapshot],
    ) -> Vec<ShardHourlySnapshot> {
        let mut merged: HashMap<u32, ShardHourlySnapshot> = existing
            .into_iter()
            .map(|shard| (shard.shard_id, shard))
            .collect();

        for shard in incoming {
            let incoming = ShardHourlySnapshot {
                shard_id: shard.shard_id,
                worst_status: shard.status,
                cumulative_latency_ms: shard.latency_ms,
                latency_sample_count: shard.latency_ms.map(|_| 1),
                last_guild_count: shard.guild_count,
                sample_count: 1,
            };

            merged
                .entry(incoming.shard_id)
                .and_modify(|existing| {
                    existing.worst_status =
                        Self::worst_shard_status(&existing.worst_status, &incoming.worst_status);

                    existing.cumulative_latency_ms = Self::merge_optional_sum(
                        existing.cumulative_latency_ms,
                        incoming.cumulative_latency_ms,
                    );

                    existing.latency_sample_count = Self::merge_optional_sum(
                        existing.latency_sample_count,
                        incoming.latency_sample_count,
                    );

                    if incoming.last_guild_count.is_some() {
                        existing.last_guild_count = incoming.last_guild_count;
                    }

                    existing.sample_count =
                        existing.sample_count.saturating_add(incoming.sample_count);
                })
                .or_insert(incoming);
        }

        let mut shards: Vec<_> = merged.into_values().collect();
        shards.sort_unstable_by_key(|shard| shard.shard_id);

        shards
    }

    fn overall_status_severity(status: &OverallStatus) -> u8 {
        match status {
            OverallStatus::MajorOutage => 4,
            OverallStatus::PartialOutage => 3,
            OverallStatus::Degraded => 2,
            OverallStatus::Operational => 1,
            OverallStatus::Unknown => 0,
        }
    }

    fn worst_overall_status(existing: &OverallStatus, incoming: &OverallStatus) -> OverallStatus {
        if Self::overall_status_severity(incoming) >= Self::overall_status_severity(existing) {
            *incoming
        } else {
            *existing
        }
    }

    fn worst_shard_status(existing: &ShardStatus, incoming: &ShardStatus) -> ShardStatus {
        let severity = |status: &ShardStatus| match status {
            ShardStatus::Offline => 3,
            ShardStatus::Degraded => 2,
            ShardStatus::Operational => 1,
            ShardStatus::Unknown => 0,
        };

        if severity(incoming) >= severity(existing) {
            *incoming
        } else {
            *existing
        }
    }

    fn merge_optional_sum(existing: Option<u32>, incoming: Option<u32>) -> Option<u32> {
        match (existing, incoming) {
            (None, None) => None,
            (existing, incoming) => Some(
                existing
                    .unwrap_or_default()
                    .saturating_add(incoming.unwrap_or_default()),
            ),
        }
    }
}
