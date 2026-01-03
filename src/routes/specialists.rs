use axum::{Router, routing::{get, post, put}};
use crate::{AppState, handlers::specialists};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/", get(specialists::get_specialists))
        .route("/{id}", get(specialists::get_specialist))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/", post(specialists::create_specialist))
        .route("/{id}", put(specialists::update_specialist))
}
