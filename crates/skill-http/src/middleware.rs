//! HTTP middleware components
//!
//! This module provides middleware for the HTTP server including:
//! - Request/response logging
//! - Error handling
//! - Rate limiting (future)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::types::ApiError;

/// Custom error type that implements IntoResponse
pub struct AppError {
    pub code: StatusCode,
    pub error: ApiError,
}

impl AppError {
    pub fn new(code: StatusCode, error: ApiError) -> Self {
        Self { code, error }
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, ApiError::not_found(message))
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, ApiError::bad_request(message))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, ApiError::internal(message))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.code, Json(self.error)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!(error = %err, "Internal server error");
        Self::internal(err.to_string())
    }
}
