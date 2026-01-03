use axum::{Router, routing::{get, post, put}};
use crate::{AppState, handlers::hospitals};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(hospitals::create_hospital))
        .route("/update/{hospital_id}", put(hospitals::update_hospital))
        .route("/blood-request", get(hospitals::get_blood_requests))
        .route("/blood-request", post(hospitals::create_blood_request))
        .route("/blood-request/{hospital_id}", put(hospitals::update_blood_request))
        

}