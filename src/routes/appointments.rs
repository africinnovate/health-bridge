use axum::{Router, routing::{get, post, delete, put}};
use crate::{AppState, handlers::appointments};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(appointments::create_appointment))
        .route("/confirm/{appointment_id}", put(appointments::confirm_appointment))
        .route("/reschedule/{appointment_id}", put(appointments::reschedule_appointment))
        .route("/cancel/{appointment_id}", put(appointments::cancel_appointment))
        .route("/complete/{appointment_id}", put(appointments::complete_appointment))
        

}