use crate::{AppState, handlers::admin};
use axum::{
    Router,
    routing::{delete, get, patch, put},
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
}
