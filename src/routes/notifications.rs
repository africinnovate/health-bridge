use axum::{Router, routing::{get, patch}};
use crate::{AppState, handlers::notifications};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(notifications::get_notifications))
        .route("/{id}/read", patch(notifications::mark_notification_as_read))
        .route("/read-all", patch(notifications::mark_all_notifications_as_read))
        

}