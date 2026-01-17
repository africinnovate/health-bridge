use crate::{AppState, handlers::specialists};
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
}
