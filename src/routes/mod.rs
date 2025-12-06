use axum::Router;

use crate::AppState;

pub mod users;
pub mod hospitals;
pub mod specialists;
pub mod patients;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/users", users::router())
        .nest("/patients", patients::router())
        .nest("/specialists", specialists::router())
        .nest("/hospitals", hospitals::router())
}
