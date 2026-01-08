use axum::{Router, routing::{get, post}};
use crate::{AppState, handlers::{auth, socials}};

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/verify-token", post(auth::verify_token))
        .route("/verify-email", post(auth::verify_email))
        .route("/social-login", post(socials::social_login))
        
    }

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/delete-account", post(auth::delete_account))
        .route("/refresh-token", post(auth::refresh_token))
        .route("/logout", post(auth::logout))
        .route("/logout-all", post(auth::logout_all_devices))

}