use std::future::{Ready, ready};

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{self, HeaderValue},
};
use futures::future::LocalBoxFuture;
use tracing::{info, warn};

use crate::{
    domain::auth::{AuthContext, AuthType, Authorization, decode_access_token},
    utils::{discord::is_valid_snowflake, logger::LogCode},
};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());
        if let Some(auth_header) = auth_header
            && let Some(authorization) = Authorization::parse(auth_header)
        {
            match authorization.auth_type {
                AuthType::Admin | AuthType::User => {
                    match decode_access_token(&authorization.token) {
                        Ok(claims) => {
                            let sub = claims.sub;
                            let sid = claims.sid;

                            info!(
                              code = %LogCode::Auth,
                              session_id = %sid,
                              "Decoded JWT for user_id: {} with auth type: {:?}",
                              sub,
                              authorization.auth_type
                            );

                            let new_auth = format!("{} {}", authorization.auth_type, sub);

                            let auth_context = AuthContext::new(authorization.auth_type)
                                .with_user_id(sub)
                                .with_session_id(sid)
                                .with_token(authorization.token.clone());

                            req.extensions_mut().insert(auth_context);

                            if let Ok(header_value) = HeaderValue::from_str(&new_auth) {
                                req.headers_mut()
                                    .insert(header::AUTHORIZATION, header_value);
                            }
                        }
                        Err(e) => {
                            warn!(
                              code = %LogCode::Auth,
                              "Failed to decode JWT: {:?}",
                              e
                            );
                        }
                    }
                }
                AuthType::Bot => {
                    let bot_id = extract_bot_id_from_path(req.path());

                    let mut auth_context = AuthContext::new(authorization.auth_type)
                        .with_token(authorization.token.clone());

                    if let Some(bot_id) = bot_id {
                        auth_context = auth_context.with_bot_id(bot_id);
                    }

                    req.extensions_mut().insert(auth_context);
                }
                _ => {
                    let auth_context = AuthContext::new(authorization.auth_type);
                    req.extensions_mut().insert(auth_context);
                }
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

fn extract_bot_id_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    for (i, segment) in segments.iter().enumerate() {
        if *segment == "bots" && i + 1 < segments.len() {
            let potential_id = segments[i + 1];
            if is_valid_snowflake(potential_id) {
                return Some(potential_id.to_string());
            }
        }
    }
    None
}
