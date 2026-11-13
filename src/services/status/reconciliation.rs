use mongodb::bson::DateTime;

use crate::{domain::error::ApiResult, repository::Repositories};

use super::snapshot::create_status_snapshot;

pub async fn reconcile_stale_heartbeats(repos: &Repositories, cutoff: DateTime) -> ApiResult<()> {
    let bot_ids = repos
        .status_current
        .find_bots_with_stale_heartbeats(cutoff)
        .await?;

    for bot_id in bot_ids {
        let mut session = repos.client.start_session().await?;

        let status_current = repos.status_current.clone();
        let status_check = repos.status_check.clone();

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
                    .create_check_with_session(snapshot, session)
                    .await?;

                Ok(())
            })
            .await?;
    }

    Ok(())
}
