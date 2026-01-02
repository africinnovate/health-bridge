use axum::Extension;
use axum::extract::Path;
use axum::{
    extract::State,
    Json,
};
use uuid::Uuid;
use serde::{Deserialize};
use diesel::prelude::*;
use utoipa::ToSchema;

use crate::models::User;
use crate::specialists::service::{
    create_specialist,
    get_specialist_with_user, update_specialist,
    SpecialistResponse,
};
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


#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
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

/// Create specialist profile
/// 
/// Creates a new specialist profile for a user.

#[utoipa::path(
    post,
    path = "/api/specialists",
    request_body = CreateSpecialistRequest,
    responses(
        (status = 200, body = ApiResponse<SpecialistResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn create_specialist_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateSpecialistRequest>,
) -> Result<ApiResponse<SpecialistResponse>, AppError> {

    let mut conn = state.pool.get()?;

    let specialist = create_specialist(
        &mut conn,
        &user,
        payload,
    )?;

    let response =
        get_specialist_with_user(&mut conn, specialist.id)?;

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
        (status = 200, body = ApiResponse<SpecialistResponse>),
        (status = 404),
        (status = 500)
    ),
    tag = "specialists"
)]
pub async fn get_specialist_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<SpecialistResponse>, AppError> {

    let mut conn = state.pool.get()?;

    let specialist =
    get_specialist_with_user(&mut conn, id)?;

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
        (status = 200, body = ApiResponse<SpecialistResponse>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn update_specialist_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateSpecialistRequest>,
) -> Result<ApiResponse<SpecialistResponse>, AppError> {

    let mut conn = state.pool.get()?;

    update_specialist(
        &mut conn,
        id,
        &user,
        payload,
    )?;

    let response = get_specialist_with_user(&mut conn, id)?;

    Ok(ApiResponse::success(response))
}
