use actix_web::web::{self, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::info;

use crate::{
    domain::error::ApiResult, openapi::schemas::AchievementResponse, repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all achievements",
    description = "Retrieve a list of all achievements in the system",
    tag = "Achievements"
)]
async fn get_achievements(
    repos: web::Data<Repositories>,
) -> ApiResult<Json<Vec<AchievementResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all achievements",
    );

    let achievements = repos.achievements.find_all_shared().await?;

    let achievement_reponses = achievements
        .into_iter()
        .map(AchievementResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        "All achievements fetched successfully",
    );

    Ok(Json(achievement_reponses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/achievements", get().to(get_achievements));
}
