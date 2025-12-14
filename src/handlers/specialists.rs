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
    path = "/specialists",
    responses(
        (status = 200, description = "List of specialists", body = MessageResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "specialists"
)]
pub async fn get_specialists(
    State(_state): State<AppState>,
) -> Result<Json<MessageResponse>, AppError> {
    Ok(Json(MessageResponse {
        message: "List of specialists".to_string(),
    }))
}
