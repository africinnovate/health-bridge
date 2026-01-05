use axum::{Router, routing::{get, post}};
use crate::{AppState, handlers::{auth, socials}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(auth::get_users))
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/verify-token", post(auth::verify_token))
        .route("/verify-email", post(auth::verify_email))
        .route("/social-login", post(socials::social_login))
}