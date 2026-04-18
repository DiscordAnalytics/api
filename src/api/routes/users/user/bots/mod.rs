use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, get},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{BotResponse, UserBotsResponse},
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get user's bots",
    description = "Fetch a list of bots owned by the authenticated user",
    tag = "Users"
)]
async fn get_user_bots(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<UserBotsResponse>> {
    let user_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        user_id = %user_id,
        "Received request to fetch user's bots"
    );

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            user_id = %user_id,
            "Admin access granted for user bots"
        );
    } else if ctx.is_user() {
        if ctx.user_id.as_deref() != Some(&user_id) {
            warn!(
                code = %LogCode::Forbidden,
                user_id = %user_id,
                "User attempted to access another user's bots"
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            user_id = %user_id,
            "Unauthenticated access attempt to user bots"
        );
        return Err(ApiError::Forbidden);
    }

    let user_bots = repos.bots.find_by_user_id(&user_id).await?;

    let (owned_bots, team_bots) = user_bots.into_iter().try_fold(
        (Vec::new(), Vec::new()),
        |(mut owned, mut team), b| -> ApiResult<(Vec<BotResponse>, Vec<BotResponse>)> {
            if b.owner_id == user_id {
                owned.push(BotResponse::try_from(b)?);
            } else if b.team.contains(&user_id) {
                let mut res = BotResponse::try_from(b)?;
                res.webhooks_config = None;
                team.push(res);
            }
            Ok((owned, team))
        },
    )?;

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
