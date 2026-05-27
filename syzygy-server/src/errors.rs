use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use syzygy_core::domain::DomainError;

#[derive(Debug)]
pub enum ApiError {
    Domain(DomainError),
    Internal(String),
    Unauthorized,
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        ApiError::Domain(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Domain(e) => match e {
                DomainError::UserNotFound => (StatusCode::NOT_FOUND, e.to_string()),
                DomainError::PostNotFound => (StatusCode::NOT_FOUND, e.to_string()),
                DomainError::UsernameAlreadyExists => (StatusCode::CONFLICT, e.to_string()),
                DomainError::InvalidCredentials => (StatusCode::UNAUTHORIZED, e.to_string()),
                DomainError::AlreadyFollowing => (StatusCode::CONFLICT, e.to_string()),
                DomainError::NotFollowing => (StatusCode::CONFLICT, e.to_string()),
                DomainError::CannotFollowSelf => (StatusCode::BAD_REQUEST, e.to_string()),
                DomainError::ContentTooLong => (StatusCode::BAD_REQUEST, e.to_string()),
                DomainError::EmptyContent => (StatusCode::BAD_REQUEST, e.to_string()),
                DomainError::Unauthorized => (StatusCode::FORBIDDEN, e.to_string()),
            },
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
            }
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
