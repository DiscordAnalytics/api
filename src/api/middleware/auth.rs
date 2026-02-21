use std::future::{Ready, ready};

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::{self, HeaderValue},
};
use futures::future::LocalBoxFuture;
use tracing::{info, warn};

use crate::{
    domain::auth::{AuthContext, AuthType, Authorization, decode_jwt},
    utils::logger::LogCode,
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
        if let Some(auth_header) = auth_header {
            if let Some(authorization) = Authorization::parse(auth_header) {
                if matches!(authorization.auth_type, AuthType::Admin | AuthType::User) {
                    match decode_jwt(&authorization.token) {
                        Ok(claims) => {
                            let sub = claims.sub;

                            info!(
                              code = %LogCode::Auth,
                              "Decoded JWT for user_id: {} with auth type: {:?}",
                              sub,
                              authorization.auth_type
                            );

                            let new_auth = format!("{} {}", authorization.auth_type, sub);

                            let auth_context =
                                AuthContext::new(authorization.auth_type).with_user_id(sub);

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
                } else {
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
