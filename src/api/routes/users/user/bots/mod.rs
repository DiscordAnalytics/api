use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::{info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{BotResponse, UserBotsResponse},
    repository::Repositories,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get user's bots",
    description = "Fetch a list of bots owned by the authenticated user",
    tag = "Users"
)]
async fn get_user_bots(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<UserBotsResponse>> {
    let user_id = id.0;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to fetch user's bots"
    );

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            user_id = %user_id,
            "Admin access granted for user bots"
        );
    } else if ctx.is_user() && ctx.user_id.as_deref() != Some(&user_id) {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "User attempted to access another user's details"
        );
        return Err(ApiError::Forbidden);
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to user details"
        );
        return Err(ApiError::Forbidden);
    }

    let user_bots = repos.bots.find_by_user_id(&user_id).await?;

    let owned_bots = user_bots
        .iter()
        .filter(|b| b.owner_id == user_id)
        .cloned()
        .map(BotResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let team_bots = user_bots
        .into_iter()
        .filter(|b| b.team.contains(&user_id))
        .map(BotResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        owned_bots_count = owned_bots.len(),
        team_bots_count = team_bots.len(),
        "Fetched user's bots"
    );

    Ok(Json(UserBotsResponse {
        owned_bots,
        team_bots,
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.route("/bots", get().to(get_user_bots));
}
