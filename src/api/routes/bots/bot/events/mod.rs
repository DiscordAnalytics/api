mod event;

use actix_web::web::{Data, Json};
use apistos::{
    api_operation,
    web::{ServiceConfig, get, post, resource, scope},
};
use tracing::{info, warn};

use crate::{
    api::middleware::{Authenticated, Snowflake},
    domain::{
        error::{ApiError, ApiResult},
        models::CustomEvent,
    },
    openapi::schemas::{CustomEventBody, CustomEventResponse},
    repository::Repositories,
    services::Services,
    utils::logger::LogCode,
};

#[api_operation(
    summary = "Get All Custom Events",
    description = "Retrieve a list of all custom events associated with the authenticated bot. This endpoint allows you to view all the custom events that have been created for your bot, including their event keys and associated graph names. Use this information to manage and organize your bot's custom events effectively.",
    tag = "Bots"
)]
async fn get_all_events(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    id: Snowflake,
) -> ApiResult<Json<Vec<CustomEventResponse>>> {
    let bot_id = id.0;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Retrieving all custom events for bot",
    );

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for retrieving all custom events",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to retrieve custom events for a different bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User attempting to retrieve custom events for a bot they don't have access to",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Unauthenticated request attempting to retrieve custom events",
        );
        return Err(ApiError::Forbidden);
    }

    let events = repos.custom_events.find_by_bot_id(&bot_id).await?;

    let event_responses = events.into_iter().map(CustomEventResponse::from).collect();

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        "Successfully retrieved all custom events for bot",
    );

    Ok(Json(event_responses))
}

#[api_operation(
    summary = "Create Custom Event",
    description = "Create a new custom event for the authenticated bot. This endpoint allows you to define a new custom event by providing an event key and an associated graph name. Custom events can be used to trigger specific actions or workflows within your bot, enabling you to create more dynamic and interactive experiences for your users.",
    tag = "Bots"
)]
async fn create_event(
    auth: Authenticated,
    services: Data<Services>,
    repos: Data<Repositories>,
    event: Json<CustomEventBody>,
    id: Snowflake,
) -> ApiResult<Json<CustomEventResponse>> {
    let bot_id = id.0;

    let bot = repos.bots.find_by_id(&bot_id).await?.ok_or_else(|| {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            "Bot not found",
        );
        ApiError::NotFound(format!("Bot with ID {} not found", bot_id))
    })?;

    if bot.suspended {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Access denied for suspended bot team",
        );
        return Err(ApiError::BotSuspended);
    }

    let ctx = &auth.0;

    if ctx.is_admin() {
        info!(
            code = %LogCode::AdminAction,
            bot_id = %bot_id,
            "Admin access granted for creating custom event",
        );
    } else if ctx.is_bot() && ctx.bot_id.as_deref() != Some(&bot_id) {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Bot attempting to create custom event for a different bot",
        );
        return Err(ApiError::Forbidden);
    } else if ctx.is_user() {
        let user_id = ctx.user_id.as_deref().ok_or(ApiError::Unauthorized)?;
        if !services.auth.user_has_bot_access(user_id, &bot_id).await? {
            warn!(
                code = %LogCode::Forbidden,
                bot_id = %bot_id,
                user_id = %user_id,
                "User attempting to create custom event for a bot they don't have access to",
            );
            return Err(ApiError::Forbidden);
        }
    } else {
        warn!(
            code = %LogCode::Forbidden,
            bot_id = %bot_id,
            "Unauthenticated request attempting to create custom event",
        );
        return Err(ApiError::Forbidden);
    }

    if repos
        .custom_events
        .find_by_bot_id_and_event_key(&bot_id, &event.event_key)
        .await?
        .is_some()
    {
        info!(
            code = %LogCode::Request,
            bot_id = %bot_id,
            event_key = %event.event_key,
            "Custom event with the same event key already exists",
        );
        return Err(ApiError::AlreadyExists(format!(
            "Custom event with event key '{}' already exists for this bot",
            event.event_key
        )));
    }

    let new_event = CustomEvent {
        bot_id: bot_id.clone(),
        default_value: event.default_value,
        event_key: event.event_key.clone(),
        graph_name: event.graph_name.clone(),
    };

    repos.custom_events.insert(&new_event).await?;

    info!(
        code = %LogCode::Request,
        bot_id = %bot_id,
        event_key = %event.event_key,
        "Custom event created successfully",
    );

    Ok(Json(CustomEventResponse::from(new_event)))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/events")
            .service(
                resource("")
                    .route(get().to(get_all_events))
                    .route(post().to(create_event)),
            )
            .configure(event::configure),
    );
}
