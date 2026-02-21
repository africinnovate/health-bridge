use crate::{AppState, handlers::hospitals};
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(hospitals::get_hospitals))
        .route("/{hospital_id}", get(hospitals::get_hospital_by_id))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/{hospital_id}", delete(hospitals::delete_hospital))
        .route("/create", post(hospitals::create_hospital))
        .route("/me", get(hospitals::get_user_hospitals))
        .route("/update/{hospital_id}", put(hospitals::update_hospital))
        .route("/blood-request", get(hospitals::get_blood_requests))
        .route("/blood-request", post(hospitals::create_blood_request))
        .route(
            "/blood-request/{hospital_id}",
            put(hospitals::update_blood_request),
        )
        .route(
            "/settings/{hospital_id}",
            get(hospitals::get_hospital_settings),
        )
        .route(
            "/settings/{hospital_id}",
            patch(hospitals::update_hospital_settings),
        )
        .route(
            "/upload-accreditation/{hospital_id}",
            post(hospitals::upload_accreditation_doc),
        )
}
