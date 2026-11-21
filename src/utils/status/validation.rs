use crate::domain::{
    error::{ApiError, ApiResult},
    models::{AnnouncementStatus, AnnouncementType, BotDeploymentMode},
};

pub(crate) const MAX_TOTAL_SHARDS: u32 = 1000;

pub(crate) fn validate_status_config(
    deployment_mode: BotDeploymentMode,
    total_shards: u32,
) -> ApiResult<()> {
    match deployment_mode {
        BotDeploymentMode::Single if total_shards != 1 => Err(ApiError::BadRequest(
            "Single deployments must have exactly one shard".into(),
        )),
        BotDeploymentMode::Sharded if total_shards == 0 => Err(ApiError::BadRequest(
            "Sharded deployments must have at least one shard".into(),
        )),
        BotDeploymentMode::Sharded if total_shards > MAX_TOTAL_SHARDS => Err(ApiError::BadRequest(
            format!("Sharded deployments must have at most {MAX_TOTAL_SHARDS} shards"),
        )),
        _ => Ok(()),
    }
}

pub(crate) fn is_valid_announcement_status(
    status: AnnouncementStatus,
    incident_type: AnnouncementType,
) -> bool {
    match (status, incident_type) {
        (
            AnnouncementStatus::Investigating
            | AnnouncementStatus::Identified
            | AnnouncementStatus::Monitoring
            | AnnouncementStatus::Resolved,
            AnnouncementType::Incident,
        ) => true,
        (
            AnnouncementStatus::Scheduled
            | AnnouncementStatus::InProgress
            | AnnouncementStatus::Completed
            | AnnouncementStatus::Cancelled,
            AnnouncementType::Maintenance,
        ) => true,
        (_, _) => false,
    }
}
