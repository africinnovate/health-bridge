use axum::{Router, routing::{get, post, put}};
use crate::{AppState, handlers::specialists};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(specialists::create_specialist))
        .route("/{id}", get(specialists::get_specialist))
        .route("/{id}", put(specialists::update_specialist))
        

}
