use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    /// Invalid timezone name
    InvalidTimezone(String),
    /// System time error
    SystemTimeError,
    /// Chrony unavailable or error
    ChronyError(String),
    /// Internal server error
    Internal(String),
    /// Timeout error
    Timeout,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidTimezone(tz) => write!(f, "Unrecognized time zone '{}'", tz),
            ApiError::SystemTimeError => write!(f, "System time error"),
            ApiError::ChronyError(msg) => write!(f, "Chrony error: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::Timeout => write!(f, "Request timeout"),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::InvalidTimezone(ref tz) => {
                (StatusCode::BAD_REQUEST, format!("Unrecognized time zone '{}'", tz))
            }
            ApiError::SystemTimeError => {
                (StatusCode::SERVICE_UNAVAILABLE, "System time error".to_string())
            }
            ApiError::ChronyError(_) => {
                // Chrony errors don't fail the request, they just mean no quality metrics
                // This shouldn't normally be converted to a response
                (StatusCode::INTERNAL_SERVER_ERROR, "Chrony error".to_string())
            }
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            ApiError::Timeout => {
                (StatusCode::REQUEST_TIMEOUT, "Request timeout".to_string())
            }
        };

        let body = Json(json!({
            "detail": message
        }));

        (status, body).into_response()
    }
}

impl From<std::time::SystemTimeError> for ApiError {
    fn from(_: std::time::SystemTimeError) -> Self {
        ApiError::SystemTimeError
    }
}

impl From<chrono_tz::ParseError> for ApiError {
    fn from(err: chrono_tz::ParseError) -> Self {
        ApiError::InvalidTimezone(err.to_string())
    }
}
