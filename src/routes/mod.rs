use axum::{Router, middleware};
use crate::AppState;
use crate::middleware::auth::require_auth;

pub mod auth;
pub mod hospitals;
pub mod specialists;
pub mod patients;
pub mod appointments;
pub mod notifications;
pub mod admin;
pub mod user_settings;

pub fn create_router() -> Router<AppState> {
    let protected_patients = Router::new()
        .nest("/patients", patients::router())
        .route_layer(
            middleware::from_fn_with_state(
                // Axum injects AppState here automatically
                |state: &AppState| state.clone(),
                require_auth,
            ),
        );

    let protected_hospitals = Router::new()
        .nest("/hospitals", hospitals::router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

    let protected_appointments = Router::new()
        .nest("/appointments", appointments::router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

    let protected_user_settings = Router::new()
        .nest("/user-settings", user_settings::router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

        let protected_notifications = Router::new()
        .nest("/notifications", notifications::router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

        let protected_specialists = Router::new()
        .nest("/specialists", specialists::protected_router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

        let public_specialists = Router::new()
        .nest("/specialists", specialists::public_router());

        let protected_admin = Router::new()
        .nest("/admin", admin::router())
        .route_layer(
            middleware::from_fn_with_state(|state: &AppState| state.clone(), require_auth),
        );

    Router::new()
        .nest("/auth", auth::router())
        .merge(public_specialists)
        .merge(protected_specialists)
        .merge(protected_patients)
        .merge(protected_hospitals)
        .merge(protected_appointments)
        .merge(protected_notifications)
        .merge(protected_user_settings)
        .merge(protected_admin)
}
