use mongodb::bson::DateTime;

use crate::{
    domain::error::ApiResult,
    domain::models::{ShardStatusSnapshot, StatusCheck},
    repository::Repositories,
    repository::status_check_hourly::StatusCheckHourlyRepository,
};

use super::snapshot::create_status_snapshot;

pub async fn reconcile_stale_heartbeats(repos: &Repositories, cutoff: DateTime) -> ApiResult<()> {
    let bot_ids = repos
        .status_current
        .find_bots_with_stale_heartbeats(cutoff)
        .await?;

    for bot_id in bot_ids {
        // Run the transaction and capture the finalized snapshot via return value.
        let finalized = run_expiry_transaction(repos, &bot_id, cutoff).await?;

        // After the transaction commits, upsert into the hourly bucket (best-effort).
        if let Some(snapshot) = finalized {
            let now = DateTime::now();
            let hour_ts =
                StatusCheckHourlyRepository::hour_timestamp(snapshot.timestamp.timestamp_millis());
            let shards: Vec<ShardStatusSnapshot> = snapshot.shards.clone();

            if let Err(err) = repos
                .status_check_hourly
                .upsert_from_check(&snapshot.bot_id, snapshot.clone(), &shards, hour_ts, now)
                .await
            {
                tracing::warn!(
                    bot_id = %snapshot.bot_id,
                    error = %err,
                    "Failed to upsert hourly bucket after reconciliation"
                );
            }
        }
    }

    Ok(())
}

/// Run the expiry transaction for a single bot. Returns the finalized snapshot
/// if any shard was actually marked offline, `None` otherwise.
async fn run_expiry_transaction(
    repos: &Repositories,
    bot_id: &str,
    cutoff: DateTime,
) -> ApiResult<Option<StatusCheck>> {
    let bot_id = bot_id.to_string();
    let mut session = repos.client.start_session().await?;

    let status_current = repos.status_current.clone();
    let status_check = repos.status_check.clone();

    // We surface the snapshot through a shared container since `and_run2`
    // takes an async closure and we need the value after the transaction.
    use std::sync::{Arc, Mutex};
    let result: Arc<Mutex<Option<StatusCheck>>> = Arc::new(Mutex::new(None));
    let result_inner = Arc::clone(&result);

    session
        .start_transaction()
        .and_run2(async move |session| {
            let Some(current) = status_current
                .expire_stale_heartbeats(&bot_id, cutoff, session)
                .await?
            else {
                return Ok(());
            };

            let snapshot = create_status_snapshot(&current);

            status_check
                .create_check_with_session(snapshot.clone(), session)
                .await?;

            *result_inner.lock().unwrap() = Some(snapshot);

            Ok(())
        })
        .await?;

    Ok(Arc::try_unwrap(result)
        .ok()
        .and_then(|m| m.into_inner().ok())
        .flatten())
}
