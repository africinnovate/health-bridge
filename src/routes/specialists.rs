use axum::{Router, routing::{get, post, put}};
use crate::{AppState, handlers::specialists};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/specialists", post(specialists::create_specialist_handler))
        .route("/specialists/{id}", get(specialists::get_specialist_handler))
        .route("/specialists/{id}", put(specialists::update_specialist_handler))
        

}
