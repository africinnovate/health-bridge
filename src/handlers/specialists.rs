use axum::Extension;
use axum::extract::Path;
use axum::{
    extract::State,
    Json,
};
use utoipa::openapi::info;
use uuid::Uuid;
use serde::{Deserialize};
use diesel::prelude::*;
use utoipa::ToSchema;
use tracing::{info};

use crate::models::User;
use crate::specialists::service;
use crate::utils::enums::DaysOfWeekEnum;
use crate::utils::response::ApiResponse;
use crate::{AppState, error::AppError, utils::enums::ConsultationTypeEnum};
use crate::schema::{specialists};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSpecialistRequest {
    pub specialty_id: Uuid,
    pub bio: Option<String>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: ConsultationTypeEnum,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<String>,
    pub secondary_phone: Option<String>,
    pub languages_spoken: Option<String>,
    pub availabilities: Vec<CreateAvailability>,
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAvailability {
    pub day_of_week: DaysOfWeekEnum,
    pub opens_at: chrono::NaiveTime,
    pub closes_at: chrono::NaiveTime,
}


#[derive(Debug, Deserialize, ToSchema, AsChangeset)]
#[diesel(table_name = specialists)]
pub struct UpdateSpecialistRequest {
    pub bio: Option<String>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: Option<ConsultationTypeEnum>,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<String>,
    pub secondary_phone: Option<String>,
    pub languages_spoken: Option<String>,
    pub suspended: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSpecialistWithAvailability {
    pub specialist: UpdateSpecialistRequest,
    pub availabilities: Option<Vec<CreateAvailability>>,
}

/// Create specialist profile
/// 
/// Creates a new specialist profile for a user.

#[utoipa::path(
    post,
    path = "/api/specialists",
    request_body = CreateSpecialistRequest,
    responses(
        (status = 200, body = ApiResponse<service::SpecialistResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn create_specialist(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateSpecialistRequest>,
) -> Result<ApiResponse<service::SpecialistResponse>, AppError> {

    let mut conn = state.pool.get()?;

    let specialist = service::create_specialist(
        &mut conn,
        &user,
        payload,
    )?;

    let response =
        service::get_specialist_with_user(&mut conn, specialist.id)?;

    Ok(ApiResponse::success(response))
}

/// Get specialist by ID
/// 
/// Retrieves a specialist's profile along with associated user information by their ID.
#[utoipa::path(
    get,
    path = "/api/specialists/{id}",
    params(
        ("id" = Uuid, Path, description = "Specialist ID")
    ),
    responses(
        (status = 200, body = ApiResponse<service::SpecialistResponse>),
        (status = 404),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn get_specialist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<service::SpecialistResponse>, AppError> {
info!("Handling get_specialist_handler for ID: {:?}", id);
    let mut conn = state.pool.get()?;

    let specialist =
    service::get_specialist_with_user(&mut conn, id)?;

    Ok(ApiResponse::success(specialist))
}


/// Update specialist profile
/// - Specialist can update own profile
/// - Partial updates allowed
#[utoipa::path(
    put,
    path = "/api/specialists/{id}",
    request_body = UpdateSpecialistRequest,
    responses(
        (status = 200, body = ApiResponse<service::SpecialistResponse>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn update_specialist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateSpecialistWithAvailability>,
) -> Result<ApiResponse<service::SpecialistResponse>, AppError> {

    let mut conn = state.pool.get()?;

    service::update_specialist(
        &mut conn,
        id,
        &user,
        payload,
    )?;

    let response = service::get_specialist_with_user(&mut conn, id)?;

    Ok(ApiResponse::success(response))
}
