use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::Error as ReqwestError;

use tracing::error;

use crate::utils::response::ApiResponse;

#[derive(Debug)]
pub enum AppError {
    DbError,
    UserAlreadyExists,
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    InternalServerError,
    ExternalService(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "User already exists".to_string(),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                msg,
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                msg,
            ),
            AppError::DbError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                msg,
            ),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::ExternalService(msg) => (
                StatusCode::BAD_GATEWAY,
                format!("External service error: {}", msg),
            ),
        };

        ApiResponse::message_only(status, message).into_response()
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        error!("Database error: {:?}", err);
        AppError::DbError
    }
}

impl From<r2d2::Error> for AppError {
    fn from(err: r2d2::Error) -> Self {
        error!("Connection pool error: {:?}", err);
        AppError::DbError
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        error!("Internal error: {:?}", err);
        AppError::InternalServerError
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::ExternalService("OAuth provider timeout".into())
        } else {
            AppError::ExternalService(err.to_string())
        }
    }
}
