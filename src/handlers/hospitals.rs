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
    path = "/hospitals",
    responses(
        (status = 200, description = "List of hospitals", body = MessageResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_hospitals(
    State(_state): State<AppState>,
) -> Result<Json<MessageResponse>, AppError> {
    Ok(Json(MessageResponse {
        message: "List of hospitals".to_string(),
    }))
}
