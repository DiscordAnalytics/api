use actix_web::web::{Data, Json};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::info;

use crate::{
    api::middleware::OptionalAuth, domain::error::ApiResult, openapi::schemas::AchievementResponse,
    repository::Repositories, utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all achievements",
    description = "Retrieve a list of all achievements in the system",
    tag = "Achievements"
)]
async fn get_achievements(
    auth: OptionalAuth,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<AchievementResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all achievements",
    );

    let is_admin = auth.as_ref().is_some_and(|ctx| ctx.is_admin());

    let achievements = repos.achievements.find_all_shared().await?;

    let reponses = achievements
        .into_iter()
        .map(|a| AchievementResponse::from_shared(a, is_admin))
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All achievements fetched successfully",
    );

    Ok(Json(reponses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/achievements", get().to(get_achievements));
}
