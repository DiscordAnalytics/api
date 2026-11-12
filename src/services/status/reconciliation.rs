use mongodb::bson::DateTime;

use crate::{domain::error::ApiResult, repository::Repositories};

use super::snapshot::create_status_snapshot;

pub async fn reconcile_stale_heartbeats(repos: &Repositories, cutoff: DateTime) -> ApiResult<()> {
    let updated_statuses = repos.status_current.expire_stale_heartbeats(cutoff).await?;

    for current in updated_statuses {
        let snapshot = create_status_snapshot(&current);

        repos.status_check.create_check(snapshot).await?;
    }

    Ok(())
}
