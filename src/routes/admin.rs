use crate::{AppState, handlers::admin};
use axum::{
    Router,
    routing::{delete, get, patch},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(admin::admin_dashboard))
        .route("/users", get(admin::get_users))
        .route(
            "/specialists/{id}/status",
            patch(admin::update_specialist_status),
        )
        .route(
            "/hospitals/{id}/status",
            patch(admin::update_hospital_status),
        )
}
