use actix_web::web::{Data, Json, Path};
use apistos::{
    api_operation,
    web::{ServiceConfig, delete, get, patch, resource, scope},
};
use tracing::{error, info, warn};

use crate::{
    api::middleware::Authenticated,
    domain::error::{ApiError, ApiResult},
    openapi::schemas::{CustomEventPayload, CustomEventUpdatePayload, MessageResponse},
    repository::{CustomEventUpdate, Repositories},
    utils::{discord::Snowflake, logger::LogCode},
};

#[api_operation(
    summary = "Get Custom Event",
    description = "Retrieve details of a specific custom event associated with the authenticated bot. This endpoint allows you to view the event key and graph name of a particular custom event. Use this information to manage and organize your bot's custom events effectively.",
    tag = "Bots"
)]
async fn get_event(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<(String, String)>,
) -> ApiResult<Json<CustomEventPayload>> {
    let (id, event_key) = path.into_inner();
    let bot_id = Snowflake::try_from(id)?.into_inner();

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
            event_key = %event_key,
            "Admin access granted for retrieving custom event",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            event_key = %event_key,
            "Bot access denied for retrieving custom event",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.has_access(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                user_id = %user_id,
                bot_id = %bot_id,
                event_key = %event_key,
                "User access denied for retrieving custom event",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Unauthorized,
            bot_id = %bot_id,
            event_key = %event_key,
            "Unauthenticated access attempt for retrieving custom event",
        );
        return Err(ApiError::Unauthorized);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot",
        );
        return Err(ApiError::BotSuspended);
    }

    let event = repos
        .custom_events
        .find_by_bot_id_and_event_key(&bot_id, &event_key)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                event_key = %event_key,
                "Custom event not found",
            );
            ApiError::NotFound(format!(
                "Custom event with key {} for bot ID {} not found",
                event_key, bot_id
            ))
        })?;

    Ok(Json(CustomEventPayload::from(event)))
}

#[api_operation(
    summary = "Update Custom Event",
    description = "Update the details of a specific custom event associated with the authenticated bot. This endpoint allows you to modify the event key and graph name of a particular custom event. Use this functionality to keep your bot's custom events up-to-date and organized effectively.",
    tag = "Bots"
)]
async fn update_event(
    auth: Authenticated,
    repos: Data<Repositories>,
    body: Json<CustomEventUpdatePayload>,
    path: Path<(String, String)>,
) -> ApiResult<Json<CustomEventPayload>> {
    let (id, event_key) = path.into_inner();
    let bot_id = Snowflake::try_from(id)?.into_inner();

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
            event_key = %event_key,
            "Admin access granted for updating custom event",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            event_key = %event_key,
            "Bot access denied for updating custom event",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.has_access(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                user_id = %user_id,
                bot_id = %bot_id,
                event_key = %event_key,
                "User access denied for updating custom event",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Unauthorized,
            bot_id = %bot_id,
            event_key = %event_key,
            "Unauthenticated access attempt for updating custom event",
        );
        return Err(ApiError::Unauthorized);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    repos
        .custom_events
        .find_by_bot_id_and_event_key(&bot_id, &event_key)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                event_key = %event_key,
                "Custom event not found",
            );
            ApiError::NotFound(format!(
                "Custom event with key {} for bot ID {} not found",
                event_key, bot_id
            ))
        })?;

    let body = body.into_inner();

    let updates = CustomEventUpdate::new().with_graph_name(&body.graph_name);

    let update_result = repos
        .custom_events
        .update(&bot_id, &event_key, updates)
        .await?
        .ok_or_else(|| {
            info!(
                code = %LogCode::Request,
                bot_id = %bot_id,
                event_key = %event_key,
                "Custom event not found for update",
            );
            ApiError::DatabaseError(format!(
                "Custom event with key {} for bot ID {} not found during update",
                event_key, bot_id
            ))
        })?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        event_key = %event_key,
        "Custom event updated successfully",
    );

    Ok(Json(CustomEventPayload::from(update_result)))
}

#[api_operation(
    summary = "Delete Custom Event",
    description = "Delete a specific custom event associated with the authenticated bot. This endpoint allows you to remove a particular custom event from your bot's configuration. Use this functionality to manage and organize your bot's custom events effectively, ensuring that only relevant events are retained.",
    tag = "Bots"
)]
async fn delete_event(
    auth: Authenticated,
    repos: Data<Repositories>,
    path: Path<(String, String)>,
) -> ApiResult<Json<MessageResponse>> {
    let (id, event_key) = path.into_inner();
    let bot_id = Snowflake::try_from(id)?.into_inner();

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
            event_key = %event_key,
            "Admin access granted for deleting custom event",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            event_key = %event_key,
            "Bot access denied for deleting custom event",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !bot.has_access(user_id) {
            warn!(
                code = %LogCode::Forbidden,
                user_id = %user_id,
                bot_id = %bot_id,
                event_key = %event_key,
                "User access denied for deleting custom event",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Unauthorized,
            bot_id = %bot_id,
            event_key = %event_key,
            "Unauthenticated access attempt for deleting custom event",
        );
        return Err(ApiError::Unauthorized);
    }

    if bot.suspended && !ctx.is_admin() {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let result = repos
        .custom_events
        .delete_by_event_key(&bot_id, &event_key)
        .await?;

    if result.deleted_count == 0 {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            event_key = %event_key,
            "Custom event not found for deletion",
        );
        return Err(ApiError::NotFound(format!(
            "Custom event with key {} for bot ID {} not found",
            event_key, bot_id
        )));
    }

    if let Err(e) = repos
        .bot_stats
        .remove_event_from_stats(&bot_id, &event_key)
        .await
    {
        error!(
            code = %LogCode::System,
            bot_id = %bot_id,
            event_key = %event_key,
            "Failed to remove event from bot stats: {}",
            e,
        );
    }

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        event_key = %event_key,
        "Custom event deleted successfully",
    );

    Ok(Json(MessageResponse {
        message: format!(
            "Custom event with key {} for bot ID {} deleted successfully",
            event_key, bot_id
        ),
    }))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/{event_key}").service(
            resource("")
                .route(get().to(get_event))
                .route(patch().to(update_event))
                .route(delete().to(delete_event)),
        ),
    );
}
