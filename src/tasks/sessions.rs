use tokio::{
    spawn,
    time::{Duration, Instant, interval_at},
};
use tracing::{error, info};

use crate::{repository::Repositories, utils::logger::LogCode};

pub fn sessions_task(repos: Repositories) {
    spawn(async move {
        let period = Duration::from_secs(60 * 60);
        let start = Instant::now() + period;
        let mut interval = interval_at(start, period);

        loop {
            interval.tick().await;

            match repos.sessions.delete_expired().await {
                Ok(deleted_count) => info!(
                    code = %LogCode::Task,
                    deleted_count = %deleted_count,
                    "Deleted expired sessions",
                ),
                Err(e) => error!(
                    code = %LogCode::DbError,
                    error = %e,
                    "Failed to delete expired sessions"
                ),
            }
        }
    });
}
