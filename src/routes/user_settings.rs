use crate::{AppState, handlers::common};
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(common::get_user_settings))
        .route("/", put(common::update_user_settings))
        .route("/preference", put(common::update_consultation_preference))
        .route("/upload-image", post(common::upload_image))
}
