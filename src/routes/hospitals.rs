use axum::{Router, routing::{get, post, delete, put}};
use crate::{AppState, handlers::hospitals};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(hospitals::create_hospital))
        .route("/update", put(hospitals::update_hospital))
        

}