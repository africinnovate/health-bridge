use crate::{AppState, handlers::{admin, admin_consultations}};
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(admin::admin_dashboard))
        .route("/users", get(admin::get_users))
        .route("/users/{id}", get(admin::get_user_profile))
        .route(
            "/specialists/{id}/status",
            patch(admin::update_specialist_status),
        )
        .route(
            "/hospitals/{id}/status",
            patch(admin::update_hospital_status),
        )
        .route("/configs/{key}", put(admin::update_app_config))
        // Consultation Type routes
        .route("/consultation-types", get(admin_consultations::list_consultation_types))
        .route("/consultation-types", post(admin_consultations::create_consultation_type))
        .route("/consultation-types/{id}", put(admin_consultations::update_consultation_type))
        .route("/consultation-types/{id}", delete(admin_consultations::delete_consultation_type))
        // Consultation Benefit routes
        .route("/consultation-benefits", get(admin_consultations::list_consultation_benefits))
        .route("/consultation-benefits", post(admin_consultations::create_consultation_benefit))
        .route("/consultation-benefits/{id}", put(admin_consultations::update_consultation_benefit))
        .route("/consultation-benefits/{id}", delete(admin_consultations::delete_consultation_benefit))
        // Type-Benefit Linking
        .route("/consultation-types/{id}/benefits", get(admin_consultations::list_type_benefits))
        .route("/consultation-types/{type_id}/benefits/{benefit_id}", post(admin_consultations::link_benefit_to_type))
        .route("/consultation-types/{type_id}/benefits/{benefit_id}", delete(admin_consultations::unlink_benefit_from_type))
}
