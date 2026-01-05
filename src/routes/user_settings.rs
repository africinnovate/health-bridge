use axum::{Router, routing::{get, put}};
use crate::{AppState, handlers::common};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(common::get_user_settings))
        .route("/", put(common::update_user_settings))
        

}