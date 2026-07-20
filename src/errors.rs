use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("Unauthorized")]
    // #[response(status = StatusCode::UNAUTHORIZED, code = "UNAUTHORIZED")]
    Unauthorized,

    #[error("Forbidden")]
    // #[response(status = StatusCode::FORBIDDEN, code = "UNAUTHORIZED")]
    Forbidden,

    #[error("Not Found")]
    // #[response(status = StatusCode::NOT_FOUND, code = "NOT_FOUND")]
    NotFound,

    #[error("Bad Request")]
    // #[response(status = StatusCode::BAD_REQUEST, code = "BAD_REQUEST")]
    BadRequest(String),

    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) | Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Sqlx(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            _ => "INTERNAL_ERROR",
        };

        let message = match self {
            Self::Unauthorized => "Unauthorized".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::NotFound => "Not Found".to_string(),
            Self::BadRequest(msg) => msg.clone(),
            Self::Validation(err) => err.to_string(),
            Self::Sqlx(_) | Self::Internal(_) => "Internal server error".to_string(),
        };

        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "code": code,
            "status": "error",
            "message": message,
        }))
    }
}
