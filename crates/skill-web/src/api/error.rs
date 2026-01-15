//! API error types

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// API error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ApiError {
    /// Network error (failed to send request)
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP error with status code
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    /// Resource not found (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Unauthorized (401)
    #[error("Unauthorized")]
    Unauthorized,

    /// Forbidden (403)
    #[error("Forbidden")]
    Forbidden,

    /// Server error (5xx)
    #[error("Server error: {0}")]
    Server(String),

    /// Timeout
    #[error("Request timed out")]
    Timeout,

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

impl ApiError {
    /// Create an error from HTTP status code and message
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            400 => Self::BadRequest(message),
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound(message),
            408 => Self::Timeout,
            500..=599 => Self::Server(message),
            _ => Self::Http { status, message },
        }
    }

    /// Check if the error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout | Self::Server(_)
        )
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound(_)
                | Self::BadRequest(_)
                | Self::Unauthorized
                | Self::Forbidden
                | Self::Validation(_)
        )
    }

    /// Get the HTTP status code if applicable
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Http { status, .. } => Some(*status),
            Self::NotFound(_) => Some(404),
            Self::BadRequest(_) => Some(400),
            Self::Unauthorized => Some(401),
            Self::Forbidden => Some(403),
            Self::Server(_) => Some(500),
            Self::Timeout => Some(408),
            _ => None,
        }
    }
}

/// API error response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<ApiErrorResponse> for ApiError {
    fn from(resp: ApiErrorResponse) -> Self {
        match resp.code.as_str() {
            "NOT_FOUND" => Self::NotFound(resp.message),
            "BAD_REQUEST" => Self::BadRequest(resp.message),
            "VALIDATION_ERROR" => Self::Validation(resp.message),
            "UNAUTHORIZED" => Self::Unauthorized,
            "FORBIDDEN" => Self::Forbidden,
            "INTERNAL_ERROR" => Self::Server(resp.message),
            _ => Self::Http {
                status: 0,
                message: resp.message,
            },
        }
    }
}

/// Result type alias for API operations
pub type ApiResult<T> = Result<T, ApiError>;
