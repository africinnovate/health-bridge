use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub async fn get_hospitals(
    State(_state): State<AppState>,
) -> Result<Json<MessageResponse>, StatusCode> {
    Ok(Json(MessageResponse {
        message: "List of hospitals".to_string(),
    }))
}