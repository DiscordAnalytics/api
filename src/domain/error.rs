use std::fmt;

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use anyhow::Error as AnyError;
use apistos::ApiErrorComponent;
use mongodb::error::Error as MongoError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ApiErrorComponent)]
#[openapi_error(
    status(
        code = 400,
        description = "Bad request due to invalid input or validation errors"
    ),
    status(
        code = 401,
        description = "Unauthorized access due to missing or invalid authentication"
    ),
    status(
        code = 403,
        description = "Forbidden access due to insufficient permissions"
    ),
    status(code = 404, description = "Resource not found"),
    status(code = 409, description = "Conflict due to resource already existing"),
    status(code = 429, description = "Too many requests due to rate limiting"),
    status(
        code = 500,
        description = "Internal server error due to unexpected conditions"
    )
)]
pub enum ApiError {
    // Database errors
    DatabaseError(String),
    NotFound(String),
    AlreadyExists(String),

    // Auth errors
    Unauthorized,
    Forbidden,
    InvalidToken,
    MissingAuth,

    // Validation errors
    InvalidInput(String),
    ValidationError(String),

    // Business logic errors
    BotSuspended,
    UserBanned,
    LimitExceeded,

    // External service errors
    StorageError(String),
    WebhookError(String),

    // Generic
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ApiError::NotFound(resource) => write!(f, "{} not found", resource),
            ApiError::AlreadyExists(resource) => write!(f, "{} already exists", resource),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Forbidden => write!(f, "Forbidden"),
            ApiError::InvalidToken => write!(f, "Invalid authentication token"),
            ApiError::MissingAuth => write!(f, "Authentication required"),
            ApiError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ApiError::BotSuspended => write!(f, "Bot is suspended"),
            ApiError::UserBanned => write!(f, "User is banned"),
            ApiError::LimitExceeded => write!(f, "Rate limit exceeded"),
            ApiError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            ApiError::WebhookError(msg) => write!(f, "Webhook error: {}", msg),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Unauthorized | ApiError::InvalidToken | ApiError::MissingAuth => {
                StatusCode::UNAUTHORIZED
            }
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::InvalidInput(_) | ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::AlreadyExists(_) => StatusCode::CONFLICT,
            ApiError::LimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string(),
            "status": self.status_code().as_u16(),
        }))
    }
}

impl From<MongoError> for ApiError {
    fn from(err: MongoError) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<AnyError> for ApiError {
    fn from(err: AnyError) -> Self {
        ApiError::InternalError(err.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
