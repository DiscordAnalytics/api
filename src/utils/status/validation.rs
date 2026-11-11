use crate::domain::{
    error::{ApiError, ApiResult},
    models::BotDeploymentMode,
};

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
        _ => Ok(()),
    }
}
