use chrono::Utc;
use mongodb::bson::DateTime;
use tokio::{
    spawn,
    time::{Duration, MissedTickBehavior, interval},
};

use crate::{
    repository::Repositories,
    services::{reconcile_stale_heartbeats, seal_expired_buckets},
    utils::{constants::STATUS_BUCKET_WINDOW_SECS, logger::LogCode},
};

/// How often to check for stale heartbeats (mark shards offline).
const RECONCILE_INTERVAL: Duration = Duration::from_secs(30);

/// How often to seal expired pending buckets into StatusChecks + hourly data.
/// Matches the heartbeat window so buckets are finalized promptly.
const SEAL_INTERVAL: Duration = Duration::from_secs(STATUS_BUCKET_WINDOW_SECS as u64);

const HEARTBEAT_TIMEOUT_MS: i64 = 15 * 60 * 1000;

pub fn status_task(repos: Repositories) {
    let repos_reconcile = repos.clone();
    let repos_seal = repos;

    // Reconciliation task (stale heartbeat -> mark offline)
    spawn(async move {
        let mut ticker = interval(RECONCILE_INTERVAL);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            let cutoff =
                DateTime::from_millis(Utc::now().timestamp_millis() - HEARTBEAT_TIMEOUT_MS);

            if let Err(error) = reconcile_stale_heartbeats(&repos_reconcile, cutoff).await {
                error!(
                    code = %LogCode::Status,
                    error = %error,
                    "Failed to reconcile stale heartbeats"
                );
            }
        }
    });

    // Sealing task (pending buckets -> StatusChecks + hourly)
    spawn(async move {
        let mut ticker = interval(SEAL_INTERVAL);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            if let Err(error) = seal_expired_buckets(&repos_seal).await {
                error!(
                    code = %LogCode::Status,
                    error = %error,
                    "Failed to seal expired status buckets"
                );
            }
        }
    });
}
