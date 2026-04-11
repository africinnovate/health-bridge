use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;
use crate::handlers::wallets as wallet_handlers;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(wallet_handlers::get_wallet_summary))
        .route("/deposit/initialize", post(wallet_handlers::deposit_initialize))
        .route("/deposit/verify/{reference}", get(wallet_handlers::deposit_verify))
        .route("/banks", get(wallet_handlers::get_banks))
        .route("/bank-accounts", post(wallet_handlers::add_bank_account))
        .route("/withdraw", post(wallet_handlers::withdraw_funds))
}
