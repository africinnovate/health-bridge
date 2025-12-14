mod db;
mod config;
mod auth;
mod models;
mod routes;
mod handlers;
mod schema;
mod docs;
mod services;
mod error;

use axum::{Router, routing::get, http::StatusCode};
use std::net::SocketAddr;
use tracing_subscriber;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env();
    let pool = db::init_db(&cfg.database_url)?;
    let mail_service = MailService::new(
        cfg.resend_api_key.clone(),
        cfg.from_email.clone(),
    );

    let state = AppState {
        pool: pool.clone(),
        cfg: cfg.clone(),
        mail_service: mail_service.clone()
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/docs")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", routes::create_router())
        .route("/health", get(|| async { (StatusCode::OK, "The health is healthing! ...") }))
        .with_state(state);

    let addr: SocketAddr = cfg.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);
    println!("Server starting on http://{}", addr);
    println!("📚 Swagger UI available at http://{}/docs", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
