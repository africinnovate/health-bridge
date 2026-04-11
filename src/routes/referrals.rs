use axum::{
    routing::{get, put},
    Router,
};
use crate::AppState;
use crate::handlers::referrals::{get_referral_summary, update_referral_link};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_referral_summary))
        .route("/link", put(update_referral_link))
}
