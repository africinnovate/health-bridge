use axum::{Router, middleware};
use crate::AppState;
use crate::middleware::auth::require_auth;

pub mod auth;
pub mod hospitals;
pub mod specialists;
pub mod patients;
pub mod appointments;

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

    Router::new()
        .nest("/auth", auth::router())
        .nest("/specialists", specialists::router())
        .merge(protected_patients)
        .merge(protected_hospitals)
        .merge(protected_appointments)
}
