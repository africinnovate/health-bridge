use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    DbError,
    UserAlreadyExists,
    Unauthorized,
    BadRequest,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "User already exists",
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Invalid credentials",
            ),
            AppError::DbError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ),
            AppError::BadRequest => (
                StatusCode::BAD_REQUEST,
                "Bad request",
            ),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
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
