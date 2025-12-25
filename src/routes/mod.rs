use axum::{Router, middleware};
use crate::AppState;
use crate::middleware::auth::require_auth;

pub mod auth;
pub mod hospitals;
pub mod specialists;
pub mod patients;

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

    Router::new()
        .nest("/auth", auth::router())
        .nest("/specialists", specialists::router())
        .nest("/hospitals", hospitals::router())
        .merge(protected_patients)
}
