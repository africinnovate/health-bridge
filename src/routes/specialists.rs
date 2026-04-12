use crate::{AppState, handlers::{specialists, specialist_packages}};
use axum::{
    Router,
    routing::{get, post, put, delete},
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
        .route("/consultation-types", get(specialist_packages::list_consultation_types))
        .route("/consultation-benefits", get(specialist_packages::list_consultation_benefits))
        .route("/consultation-types/{id}/benefits", get(specialist_packages::list_type_benefits))
        // Consultation Package CRUD
        .route("/packages", post(specialist_packages::create_package))
        .route("/packages", get(specialist_packages::list_packages))
        .route("/packages/{id}", get(specialist_packages::get_package))
        .route("/packages/{id}", put(specialist_packages::update_package))
        .route("/packages/{id}", delete(specialist_packages::delete_package))
}
