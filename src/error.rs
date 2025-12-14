use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
// use diesel::r2d2;

#[derive(Debug)]
pub enum AppError {
    DbError,
    UserAlreadyExists,
    Unauthorized,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                Json(json!({ "error": "User already exists" })),
            ).into_response(),

            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid credentials" })),
            ).into_response(),

            AppError::DbError | AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            ).into_response(),
        }
    }
}

// Diesel query errors
impl From<diesel::result::Error> for AppError {
    fn from(_: diesel::result::Error) -> Self {
        AppError::DbError
    }
}

// Connection pool errors (THIS FIXES ERROR #2)
impl From<r2d2::Error> for AppError {
    fn from(_: r2d2::Error) -> Self {
        AppError::DbError
    }
}

// JWT / utility errors (THIS FIXES ERROR #1)
impl From<anyhow::Error> for AppError {
    fn from(_: anyhow::Error) -> Self {
        AppError::InternalServerError
    }
}
