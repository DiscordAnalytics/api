use chrono::Utc;
use mongodb::bson::DateTime;
use tokio::{
    spawn,
    time::{Duration, MissedTickBehavior, interval},
};

use crate::{
    repository::Repositories, services::reconcile_stale_heartbeats, utils::logger::LogCode,
};

const TASK_INTERVAL: Duration = Duration::from_secs(30);
const HEARTBEAT_TIMEOUT_MS: i64 = 15 * 60 * 1000;

pub fn status_task(repos: Repositories) {
    spawn(async move {
        let mut ticker = interval(TASK_INTERVAL);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;

            let cutoff =
                DateTime::from_millis(Utc::now().timestamp_millis() - HEARTBEAT_TIMEOUT_MS);

            if let Err(error) = reconcile_stale_heartbeats(&repos, cutoff).await {
                error!(
                    code = %LogCode::Status,
                    error = %error,
                    "Failed to reconcile stale heartbeats"
                );
            }
        }
    });
}
