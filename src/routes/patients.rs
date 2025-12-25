use axum::{Router, routing::{get, delete, put}};
use crate::{AppState, handlers::patients};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/profile", get(patients::get_profile))
        .route("/profile", put(patients::update_profile))
        .route("/profile", delete(patients::delete_account))
        .route("/medical-info", put(patients::update_medical_info))
        

}