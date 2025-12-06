use axum::{Router, routing::get, extract::State, Json};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_hospitals))
}

async fn get_hospitals(State(state): State<AppState>) -> Json<&'static str> {
    Json("list of hospitals")
}
