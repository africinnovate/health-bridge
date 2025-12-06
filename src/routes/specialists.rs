use axum::{Router, routing::get, extract::State, Json};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_specialists))
}

async fn get_specialists(State(state): State<AppState>) -> Json<&'static str> {
    Json("list of specialists")
}
