use crate::infrastructure::distributed_lock::DistributedLockError;
use crate::models::SeatsError;
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not Found")]
    NotFound,

    #[error("Bad Request")]
    BadRequest(String),

    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),

    #[error(transparent)]
    DistributedLock(#[from] DistributedLockError),

    #[error(transparent)]
    SeatsError(#[from] SeatsError),

    #[error("Idempotency key was reused with a different request")]
    IdempotencyConflict,

    #[error("Invalid Idempotency-Key header")]
    InvalidIdempotencyKey,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::IdempotencyConflict => StatusCode::CONFLICT,
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Sqlx(err) => tracing::error!(?err),
            Self::Internal(err) => tracing::error!(?err),
            _ => tracing::warn!(?self),
        }

        let code = match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::IdempotencyConflict => "IDEMPOTENCY_CONFLICT",
            Self::InvalidIdempotencyKey => "INVALID_IDEMPOTENCY_KEY",
            _ => "INTERNAL_ERROR",
        };

        let message = match self {
            Self::Unauthorized => "Unauthorized".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::NotFound => "Not Found".to_string(),
            Self::BadRequest(msg) => msg.clone(),
            Self::Validation(err) => err.to_string(),
            Self::InvalidIdempotencyKey => "Invalid Idempotency-Key header".to_string(),
            Self::IdempotencyConflict => {
                "Idempotency key was reused with a different request".to_string()
            }
            _ => "Internal server error".to_string(),
        };

        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "code": code,
            "status": "error",
            "message": message,
        }))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.into())
    }
}
