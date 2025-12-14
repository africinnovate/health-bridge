use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::{AppState, error::AppError};

#[derive(Serialize, utoipa::ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/patients",
    responses(
        (status = 200, description = "List of patients", body = MessageResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "patients"
)]
pub async fn get_patients(
    State(_state): State<AppState>,
) -> Result<Json<MessageResponse>, AppError> {
    Ok(Json(MessageResponse {
        message: "List of patients".to_string(),
    }))
}
