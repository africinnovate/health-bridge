use axum::{
    Extension, Json, extract::{State, Path}
};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize};
use utoipa::ToSchema;

use crate::{
    AppState, 
    error::AppError, 
    hospitals::{
        self, appointments::CreateAppointment, 
        }, 
        models::{Appointment, User}, 
        utils::response::ApiResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct RescheduleAppointmentPayload {
    pub scheduled_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelAppointmentPayload {
    pub reason: Option<String>,
}

/// Create appointment (donor or patient initiates)
#[utoipa::path(
    post,
    path = "/api/appointments/create",
    request_body = CreateAppointment,
    responses(
        (status = 201, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn create_appointment(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateAppointment>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;

    let appointment = hospitals::appointments::create_appointment(
        &mut conn,
        &user,
        payload,
    )?;

    Ok(ApiResponse::created(
        "Appointment created successfully",
        appointment,
    ))
}


/// Confirm appointment
/// 
/// Used by Hospital staff to confirm appointment
#[utoipa::path(
    put,
    path = "/api/appointments/confirm/{appointment_id}",
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn confirm_appointment(
    Path(id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appt = hospitals::appointments::confirm_appointment(&mut conn, id, &user)?;
    Ok(ApiResponse::success(appt))
}

/// Reschedule an appointment. Used by Hospital staff
#[utoipa::path(
    put,
    path = "/api/appointments/reschedule/{appointment_id}",
    request_body = RescheduleAppointmentPayload,
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn reschedule_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(payload): Json<RescheduleAppointmentPayload>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appointment = hospitals::appointments::reschedule_appointment(
        &mut conn,
        appointment_id,
        &user,
        payload.scheduled_time,
    )?;
    Ok(ApiResponse::success(appointment))
}

/// Cancel Appointment
#[utoipa::path(
    put,
    path = "/api/appointments/cancel/{appointment_id}",
    request_body = CancelAppointmentPayload,
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn cancel_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(payload): Json<CancelAppointmentPayload>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appointment = hospitals::appointments::cancel_appointment(
        &mut conn,
        appointment_id,
        &user,
        payload.reason,
    )?;
    Ok(ApiResponse::success(appointment))
}

/// Complete appointment. Used only by Hospital
#[utoipa::path(
    put,
    path = "/api/appointments/complete/{appointment_id}",
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn complete_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appointment =
        hospitals::appointments::complete_appointment(&mut conn, appointment_id, &user)?;
    Ok(ApiResponse::success(appointment))
}
