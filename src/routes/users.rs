use axum::{Router, routing::get, extract::State, Json};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
}

async fn get_users(State(state): State<AppState>) -> Json<&'static str> {
    // use state.pool.get()? to get Diesel connection
    Json("list of users")
}
