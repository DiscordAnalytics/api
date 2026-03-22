use actix_web::rt;
use tokio::time::{Duration, interval};
use tracing::{error, info};

use crate::{repository::Repositories, utils::logger::LogCode};

pub fn invitations_task(repos: Repositories) {
    rt::spawn(async move {
        let mut interval = interval(Duration::from_secs(60 * 60));

        loop {
            interval.tick().await;

            match repos.team_invitations.delete_expired_invitations().await {
                Ok(deleted_count) => info!(
                    code = %LogCode::Server,
                    deleted_count = %deleted_count,
                    "Deleted expired team invitations",
                ),
                Err(e) => error!(
                    code = %LogCode::Server,
                    error = %e,
                    "Failed to delete expired team invitations"
                ),
            }
        }
    });
}
