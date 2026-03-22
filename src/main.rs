mod admin;
mod auth;
mod common;
mod config;
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

use axum::{Router, http::StatusCode, routing::get};
use std::net::SocketAddr;
use tracing_subscriber;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::DbPool;
use crate::docs::ApiDoc;
use crate::services::MailService;

#[derive(Clone)]
struct AppState {
    pool: DbPool,
    cfg: config::Config,
    mail_service: MailService,
    cloudinary_service: std::sync::Arc<common::services::CloudinaryService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env();
    let pool = db::init_db(&cfg.database_url)?;
    let mail_service = MailService::new(cfg.resend_api_key.clone(), cfg.from_email.clone());
    let cloudinary_service = std::sync::Arc::new(common::services::CloudinaryService::new(&cfg));

    let state = AppState {
        pool: pool.clone(),
        cfg: cfg.clone(),
        mail_service: mail_service.clone(),
        cloudinary_service,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", routes::create_router())
        .route(
            "/health",
            get(|| async { (StatusCode::OK, "The health is healthing! ...") }),
        )
        .layer(axum::Extension(state.clone()))
        .layer(cors)
        .with_state(state);

    let addr: SocketAddr = cfg.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);
    println!("Server starting on http://{}", addr);
    println!("📚 Swagger UI available at http://{}/docs", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
