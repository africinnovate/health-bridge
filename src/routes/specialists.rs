use crate::{AppState, handlers::{specialists, admin_consultations}};
use axum::{
    Router,
    routing::{get, post, put},
};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(specialists::get_specialists))
        .route("/{id}", get(specialists::get_specialist))
        .route("/specialties", get(specialists::list_specialties))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/", post(specialists::create_specialist))
        .route("/{id}", put(specialists::update_specialist))
        .route("/specialties", post(specialists::add_specialty))
        .route("/license", post(specialists::upload_license))
        .route("/patients/{id}", get(specialists::get_patient_profile))
        // Consultation routes
        .route("/consultation-types", get(admin_consultations::list_consultation_types))
        .route("/consultation-benefits", get(admin_consultations::list_consultation_benefits))
        .route("/consultation-types/{id}/benefits", get(admin_consultations::list_type_benefits))
}
