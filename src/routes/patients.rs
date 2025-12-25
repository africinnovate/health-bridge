use axum::{Router, routing::{get, delete, put}};
use crate::{AppState, handlers::patients};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get-profile/{user_id}", get(patients::get_profile))
        .route("/update-profile/{user_id}", put(patients::update_profile))
        .route("/delete-account/{user_id}", delete(patients::delete_account))
        

}