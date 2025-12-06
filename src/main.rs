mod db;
mod config;
mod auth;
mod models;
mod routes;
mod handlers;
mod schema;

use axum::{Router, routing::get, http::StatusCode};
use std::net::SocketAddr;
use tracing_subscriber;

use crate::db::DbPool;

#[derive(Clone)]
struct AppState {
    pool: DbPool,
    cfg: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env();
    let pool = db::init_db(&cfg.database_url)?;

    let state = AppState {
        pool: pool.clone(),
        cfg: cfg.clone(),
    };

    let app = Router::new()
        .nest("/api", routes::create_router())
        .route("/health", get(|| async { (StatusCode::OK, "OK") }))
        .with_state(state);

    let addr: SocketAddr = cfg.bind_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
