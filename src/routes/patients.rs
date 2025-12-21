use axum::{Router, routing::{get, post, put}, extract::State, Json};
use crate::{AppState, handlers::patients};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get-profile/{user_id}", get(patients::get_profile))
        .route("/update-profile/{user_id}", put(patients::update_profile))
}


