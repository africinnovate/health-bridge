use axum::{Router, routing::{get, post}};
use crate::{AppState, handlers::users};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(users::get_users))
        .route("/register", post(users::register))
        .route("/login", post(users::login))
        .route("/forgot-password", post(users::forgot_password))
        .route("/reset-password", post(users::reset_password))
        .route("/verify-token", post(users::verify_token))
}