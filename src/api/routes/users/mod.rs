mod user;

use actix_web::web::{Data, Json};
use anyhow::Result;
use apistos::{
    api_operation,
    web::{ServiceConfig, get, resource, scope},
};
use tracing::info;

use crate::{
    api::middleware::RequireAdmin, domain::error::ApiResult, openapi::schemas::UserResponse,
    repository::Repositories, utils::logger::LogCode,
};

#[api_operation(
    summary = "Get all users",
    description = "Fetch a list of all users registered in the Discord Analytics API",
    tag = "Users",
    skip
)]
async fn get_all_users(
    _admin: RequireAdmin,
    repos: Data<Repositories>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    info!(
        code = %LogCode::Request,
        "Fetching all users",
    );

    let users = repos.users.find_all().await?;

    let user_responses = users
        .into_iter()
        .map(UserResponse::try_from)
        .collect::<Result<Vec<_>>>()?;

    info!(
        code = %LogCode::Request,
        "All users fetched successfully",
    );

    Ok(Json(user_responses))
}

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/users")
            .service(resource("").route(get().to(get_all_users)))
            .configure(user::configure),
    );
}
