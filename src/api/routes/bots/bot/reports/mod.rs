use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, post, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::{
        error::{ApiError, ApiResult},
        models::StatsReport,
    },
    openapi::schemas::{MessageResponse, StatsReportResponse, StatsReportSubPayload},
    repository::Repositories,
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get all subscribed reports for a bot",
    description = "Get all subscribed reports for a bot",
    tag = "Reports"
)]
async fn get_subscriptions(
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
    } else if !ctx.is_user() {
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
async fn subscribe(
    auth: Authenticated,
    repos: Data<Repositories>,
    payload: Json<StatsReportSubPayload>,
    id: Path<String>,
) -> ApiResult<Json<StatsReportResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    let body = payload.into_inner();
    let user_id = body.user_id;
    let frequency = body.frequency;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Subscribing to reports for bot",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with id {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for subscribing to bot reports",
        );
    } else if ctx.is_user() {
        let authenticated_user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if authenticated_user_id != user_id || !bot.has_access(authenticated_user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to subscribe to bot reports",
            );
            return Err(ApiError::Forbidden);
        }
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied to subscribe to bot reports",
        );
        return Err(ApiError::Forbidden);
    }

    if repos
        .stats_reports
        .find_by_bot_and_user(&bot_id, &user_id, frequency.as_str())
        .await?
        .is_some()
    {
        warn!(
            code = %LogCode::Conflict,
            bot_id = %bot_id,
            user_id = %user_id,
            frequency = %frequency.as_str(),
            "Report subscription already exists",
        );
        return Err(ApiError::Conflict(
            "Report subscription already exists".to_string(),
        ));
    }

    let subscription = StatsReport::new(&bot_id, &user_id, frequency);
    repos.stats_reports.insert(&subscription).await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %user_id,
        frequency = %frequency.as_str(),
        "Report subscription created",
    );

    Ok(Json(StatsReportResponse::try_from(subscription)?))
}

#[api_operation(
    summary = "Unsubscribe from a report for a bot",
    description = "Unsubscribe from a report for a bot",
    tag = "Reports"
)]
async fn unsubscribe(
    auth: Authenticated,
    repos: Data<Repositories>,
    payload: Json<StatsReportSubPayload>,
    id: Path<String>,
) -> ApiResult<Json<MessageResponse>> {
    let bot_id = Snowflake::try_from(id.into_inner())?.into_inner();

    let body = payload.into_inner();
    let user_id = body.user_id;
    let frequency = body.frequency;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Deleting report subscription",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with id {} not found", bot_id))
    })?;

    let ctx = &auth;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for unsubscribing to bot reports",
        );
    } else if ctx.is_user() {
        let authenticated_user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if authenticated_user_id != user_id || !bot.has_access(authenticated_user_id) {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User does not have access to unsubscribe from bot reports",
            );
            return Err(ApiError::Forbidden);
        }
    } else if !ctx.is_user() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied to unsubscribe from bot reports",
        );
        return Err(ApiError::Forbidden);
    }

    if repos
        .stats_reports
        .find_by_bot_and_user(&bot_id, &user_id, frequency.as_str())
        .await?
        .is_none()
    {
        warn!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            user_id = %user_id,
            frequency = %frequency.as_str(),
            "Report subscription not found",
        );
        return Err(ApiError::NotFound(
            "Report subscription not found".to_string(),
        ));
    }

    repos
        .stats_reports
        .delete_by_bot_and_user(&bot_id, &user_id, frequency.as_str())
        .await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        user_id = %user_id,
        frequency = %frequency.as_str(),
        "Unsubscribed from bot report",
    );

    Ok(Json(MessageResponse {
        message: "Unsubscribed from bot report".to_string(),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/reports").service(
            resource("")
                .route(get().to(get_subscriptions))
                .route(post().to(subscribe))
                .route(delete().to(unsubscribe)),
        ),
    );
}
