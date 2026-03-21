use std::ops::Deref;

use actix_web::{
    Error, FromRequest, HttpMessage, HttpRequest,
    dev::Payload,
    error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized},
    web::Data,
};
use apistos::{ApiComponent, ApiSecurity};
use futures::{
    StreamExt as _,
    future::{LocalBoxFuture, Ready, ready},
};
use schemars::JsonSchema;

use crate::{
    domain::{auth::AuthContext, error::ApiError},
    services::Services,
};

#[derive(Clone, JsonSchema, ApiSecurity)]
#[openapi_security(scheme(security_type(api_key(
    name = "Authorization",
    api_key_in = "header"
))))]
pub struct Authenticated(pub AuthContext);

impl Deref for Authenticated {
    type Target = AuthContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for Authenticated {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<AuthContext>() {
            Some(context) => {
                let ctx = context.clone();

                if ctx.is_admin() {
                    let services = match req.app_data::<Data<Services>>() {
                        Some(services) => services,
                        None => {
                            return ready(Err(ErrorInternalServerError(ApiError::InternalError(
                                "Services not available".to_string(),
                            ))));
                        }
                    };

                    if let Some(user_id) = ctx.user_id.as_deref() {
                        if !services.auth.is_admin(user_id) {
                            return ready(Err(ErrorForbidden(ApiError::Forbidden)));
                        }
                    } else {
                        return ready(Err(ErrorUnauthorized(ApiError::Unauthorized)));
                    }
                }

                ready(Ok(Authenticated(ctx)))
            }
            None => ready(Err(ErrorUnauthorized(ApiError::Unauthorized))),
        }
    }
}

#[derive(Clone, JsonSchema, ApiSecurity)]
#[openapi_security(scheme(security_type(api_key(
    name = "Authorization",
    api_key_in = "header"
))))]
pub struct OptionalAuth(pub Option<AuthContext>);

impl Deref for OptionalAuth {
    type Target = Option<AuthContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for OptionalAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let context = req.extensions().get::<AuthContext>().cloned();
        ready(Ok(OptionalAuth(context)))
    }
}

#[derive(Clone, JsonSchema, ApiSecurity)]
#[openapi_security(scheme(security_type(api_key(
    name = "Authorization",
    api_key_in = "header"
))))]
pub struct RequireAdmin(pub AuthContext);

impl Deref for RequireAdmin {
    type Target = AuthContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for RequireAdmin {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ctx = match req.extensions().get::<AuthContext>() {
            Some(ctx) => ctx.clone(),
            None => return ready(Err(ErrorUnauthorized(ApiError::Unauthorized))),
        };

        let services = match req.app_data::<Data<Services>>() {
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

#[derive(Clone, JsonSchema, ApiComponent)]
pub struct RawBody(pub Vec<u8>);

impl RawBody {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl FromRequest for RawBody {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let mut payload = payload.take();
        Box::pin(async move {
            let mut bytes = Vec::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                bytes.extend_from_slice(&chunk);
            }
            Ok(RawBody(bytes))
        })
    }
}
