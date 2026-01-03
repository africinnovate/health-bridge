use axum::{Router, routing::{get, delete, put}};
use crate::{AppState, handlers::admin};

pub fn router() -> Router<AppState> {
    Router::new()
            .route("/dashboard", get(admin::admin_dashboard))
        

}