use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, post, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{MessageResponse, StatsReportResponse},
    repository::Repositories,
    services::Services,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get all subscribed reports for a bot",
    description = "Get all subscribed reports for a bot",
    tag = "Reports"
)]
async fn get_reports(
    auth: Authenticated,
    repos: Data<Repositories>,
    id: Path<String>,
) -> ApiResult<Json<Vec<StatsReportResponse>>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetching reports subscriptions for bot"
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for bot reports subscriptions",
        );
    } else if ctx.is_bot() && ctx.token.as_deref() != Some(&bot.token) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to access reports subscriptions of another bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.has_access(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to bot reports subscriptions",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for bot reports subscriptions",
        );
        return Err(ApiError::Forbidden);
    }

    let reports = repos.stats_reports.find_by_bot(&bot_id).await?;

    let reports_responses = reports
        .into_iter()
        .map(StatsReportResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Fetched reports subscriptions for bot",
    );

    Ok(Json(reports_responses))
}

#[api_operation(
    summary = "Subscribe to a report for a bot",
    description = "Subscribe to a report for a bot",
    tag = "Reports"
)]
async fn subscribe_report(auth: Authenticated) -> ApiResult<Json<MessageResponse>> {
    unimplemented!()
}

#[api_operation(
    summary = "Unsubscribe from a report for a bot",
    description = "Unsubscribe from a report for a bot",
    tag = "Reports"
)]
async fn unsubscribe_report(auth: Authenticated) -> ApiResult<Json<MessageResponse>> {
    unimplemented!()
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/reports").service(
            resource("")
                .route(get().to(get_reports))
                .route(post().to(subscribe_report))
                .route(delete().to(unsubscribe_report)),
        ),
    );
}
