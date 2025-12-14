use axum::Router;

use crate::AppState;

pub mod auth;
pub mod hospitals;
pub mod specialists;
pub mod patients;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/patients", patients::router())
        .nest("/specialists", specialists::router())
        .nest("/hospitals", hospitals::router())
}
