use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest,
    dev::Payload,
    error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
    web,
};
use apistos::ApiComponent;
use futures::future::{Ready, ready};
use schemars::JsonSchema;

use crate::{
    domain::{auth::AuthContext, error::ApiError},
    services::Services,
};

#[derive(Clone, JsonSchema, ApiComponent)]
pub struct Authenticated(pub AuthContext);

impl FromRequest for Authenticated {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthContext>() {
            Some(context) => ready(Ok(Authenticated(context.clone()))),
            None => ready(Err(ErrorUnauthorized(ApiError::Unauthorized))),
        }
    }
}

#[derive(Clone, JsonSchema, ApiComponent)]
pub struct OptionalAuth(pub Option<AuthContext>);

impl FromRequest for OptionalAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let context = req.extensions().get::<AuthContext>().cloned();
        ready(Ok(OptionalAuth(context)))
    }
}

#[derive(Clone, JsonSchema, ApiComponent)]
pub struct RequireAdmin(pub AuthContext);

impl FromRequest for RequireAdmin {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ctx = match req.extensions().get::<AuthContext>() {
            Some(ctx) => ctx.clone(),
            None => return ready(Err(ErrorUnauthorized(ApiError::Unauthorized))),
        };

        let services = match req.app_data::<web::Data<Services>>() {
            Some(services) => services,
            None => {
                return ready(Err(ErrorInternalServerError(ApiError::InternalError(
                    "Services not available".to_string(),
                ))));
            }
        };

        let is_admin = match ctx.user_id.as_deref() {
            Some(user_id) => ctx.is_admin() && services.auth.is_admin(user_id),
            None => false,
        };

        if is_admin {
            ready(Ok(RequireAdmin(ctx)))
        } else {
            ready(Err(ErrorForbidden(ApiError::Forbidden)))
        }
    }
}
