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
        .route("/nearby", get(hospitals::get_nearby_hospitals))
        .route("/update/{hospital_id}", put(hospitals::update_hospital))
        .route("/blood-request", get(hospitals::get_blood_requests))
        .route("/blood-request", post(hospitals::create_blood_request))
        .route(
            "/blood-request/{hospital_id}",
            put(hospitals::update_blood_request),
        )
        .route(
            "/blood-request/donor-stats/{donor_id}",
            get(hospitals::get_donor_stats),
        )
        .route(
            "/blood-request/donor-history/{donor_id}",
            get(hospitals::get_donor_history),
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
        .route(
            "/upload-image/{hospital_id}",
            post(hospitals::upload_hospital_image),
        )
        .route("/donors", get(hospitals::get_donors))
        .route("/donors/{donor_id}", patch(hospitals::update_donor))
        .route("/dashboard/stats", get(hospitals::get_dashboard_stats))
        .route(
            "/dashboard/recent-activity",
            get(hospitals::get_recent_activity),
        )
        .route(
            "/inventory/{hospital_id}/{blood_type}",
            patch(hospitals::update_blood_inventory),
        )
}
