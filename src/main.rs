mod admin;
mod auth;
mod common;
mod config;
mod consultations;
mod db;
mod docs;
mod error;
mod handlers;
mod hospitals;
mod middleware;
mod models;
mod patients;
mod routes;
mod schema;
mod services;
mod specialists;
mod utils;

use ax_ex::FromRef;
use common::services::CloudinaryService;
use config::Config;
use db::DbPool;
use services::mail::MailService;
use services::paystack::PaystackService;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ax_ex {
    pub use axum::extract::FromRef;
}

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub cfg: Config,
    pub mail_service: MailService,
    pub paystack_service: Arc<PaystackService>,
    pub cloudinary_service: Arc<CloudinaryService>,
}

impl FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for MailService {
    fn from_ref(state: &AppState) -> Self {
        state.mail_service.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.cfg.clone()
    }
}

impl FromRef<AppState> for Arc<PaystackService> {
    fn from_ref(state: &AppState) -> Self {
        state.paystack_service.clone()
    }
}

impl FromRef<AppState> for Arc<CloudinaryService> {
    fn from_ref(state: &AppState) -> Self {
        state.cloudinary_service.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "health_bridge=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting health-bridge server...");

    let config = Config::from_env();
    let pool = db::init_db(&config.database_url)?;
    let mail_service = MailService::new(config.resend_api_key.clone(), config.from_email.clone());
    let paystack_service = Arc::new(PaystackService::new(config.paystack_secret_key.clone()));
    let cloudinary_service = Arc::new(CloudinaryService::new(&config));

    let state = AppState {
        pool,
        cfg: config.clone(),
        mail_service,
        paystack_service,
        cloudinary_service,
    };

    let app = routes::create_router().with_state(state);

    let addr: SocketAddr = config.bind_addr.parse()?;
    info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
