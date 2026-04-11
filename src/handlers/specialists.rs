use axum::Extension;
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use diesel::prelude::*;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::{Specialty, User};
use crate::specialists::service::{
    self, CreateSpecialtyRequest, SpecialistAvailabilityResponse, SpecialistFilters,
    SpecialistResponse,
};
use crate::utils::enums::DaysOfWeekEnum;
use crate::utils::response::ApiResponse;
use crate::{AppState, error::AppError, utils::enums::ConsultationTypeEnum};

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
    pub country: Option<String>,
    pub time_zone: Option<String>,
    pub license_url: Option<String>,
    pub availabilities: Vec<CreateAvailability>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAvailability {
    pub day_of_week: DaysOfWeekEnum,
    pub opens_at: chrono::NaiveTime,
    pub closes_at: chrono::NaiveTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSpecialistRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub bio: Option<String>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: Option<ConsultationTypeEnum>,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<String>,
    pub secondary_phone: Option<String>,
    pub languages_spoken: Option<String>,
    pub time_zone: Option<String>,
    pub license_url: Option<String>,
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

    let specialist = service::create_specialist(&mut conn, &user, payload)?;

    let response = service::get_specialist_with_user(&mut conn, specialist.user_id)?;

    Ok(ApiResponse::success(response))
}

/// Get list of specialists
///
/// Retrieves a list of specialists with optional filtering by verified status, suspended status, and specialty ID
#[utoipa::path(
    get,
    path = "/api/specialists",
    params(
        ("verified" = Option<bool>, Query, description = "Filter by verified status"),
        ("suspended" = Option<bool>, Query, description = "Filter by suspended status"),
        ("specialty_id" = Option<Uuid>, Query, description = "Filter by specialty ID")
    ),
    responses(
        (status = 200, body = ApiResponse<SpecialistResponse>),
        (status = 500)
    ),
    tag = "specialists",
)]

pub async fn get_specialists(
    State(state): State<AppState>,
    Query(query): Query<SpecialistFilters>,
) -> Result<ApiResponse<Vec<SpecialistResponse>>, AppError> {
    let mut conn = state.pool.get()?;

    let rows = service::get_specialists(
        &mut conn,
        SpecialistFilters {
            verified: query.verified,
            suspended: query.suspended,
            specialty_id: query.specialty_id,
        },
    )?;

    let response = rows
        .into_iter()
        .map(|(specialist, user, availability)| SpecialistResponse {
            id: specialist.id,

            user_id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            gender: user.gender,
            city: user.city,
            state: user.state,
            image_url: user.image_url,
            email_verified: user.email_verified,
            consultation_preference: user.consultation_preference,

            hospital_id: specialist.hospital_id,
            specialty_id: specialist.specialty_id,
            bio: specialist.bio,
            years_of_experience: specialist.years_of_experience,
            consultation_type: specialist.consultation_type,
            session_duration_minutes: specialist.session_duration_minutes,
            primary_phone: specialist.primary_phone,
            secondary_phone: specialist.secondary_phone,
            languages_spoken: specialist.languages_spoken,
            license_url: specialist.license_url,
            country: specialist.country,
            time_zone: specialist.time_zone,
            verified: specialist.verified,
            suspended: specialist.suspended,
            created_at: specialist.created_at,

            availability: availability
                .into_iter()
                .map(|a| SpecialistAvailabilityResponse {
                    day_of_week: a.day_of_week,
                    opens_at: a.opens_at,
                    closes_at: a.closes_at,
                })
                .collect(),
        })
        .collect();

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
)]
pub async fn get_specialist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<service::SpecialistResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let specialist = service::get_specialist_with_user(&mut conn, id)?;

    Ok(ApiResponse::success_with_message(
        "Specialist retrieved successfully",
        specialist,
    ))
}

/// Update specialist profile
///
/// Partial updates allowed
#[utoipa::path(
    put,
    path = "/api/specialists/{id}",
    request_body = UpdateSpecialistWithAvailability,
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

    service::update_specialist(&mut conn, id, &user, payload)?;

    let response = service::get_specialist_with_user(&mut conn, user.id)?;

    Ok(ApiResponse::success_with_message(
        "Specialist updated successfully",
        response,
    ))
}

/// Add a new specialty
#[utoipa::path(
    post,
    path = "/api/specialists/specialties",
    request_body = CreateSpecialtyRequest,
    responses(
        (status = 201, body = ApiResponse<Specialty>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn add_specialty(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<service::CreateSpecialtyRequest>,
) -> Result<ApiResponse<crate::models::Specialty>, AppError> {
    use crate::utils::enums::Role;

    if matches!(user.role, Role::Donor | Role::Patient | Role::PatientDonor) {
        return Err(AppError::Unauthorized(
            "Access restricted for patients and donors".into(),
        ));
    }

    let mut conn = state.pool.get()?;
    let specialty = service::add_specialty(&mut conn, payload)?;

    Ok(ApiResponse::created(
        "Specialty added successfully",
        specialty,
    ))
}

/// List all specialties
#[utoipa::path(
    get,
    path = "/api/specialists/specialties",
    responses(
        (status = 200, body = ApiResponse<Vec<Specialty>>),
        (status = 500)
    ),
    tag = "specialists",
)]
pub async fn list_specialties(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<crate::models::Specialty>>, AppError> {
    let mut conn = state.pool.get()?;
    let specialties = service::list_specialties(&mut conn)?;

    Ok(ApiResponse::success(specialties))
}

/// Upload license
#[utoipa::path(
    post,
    path = "/api/specialists/license",
    responses(
        (status = 200, body = ApiResponse<String>),
        (status = 500)
    ),
    tag = "specialists",
)]
pub async fn upload_license(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: axum::extract::Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut mime_type = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "file" {
            filename = field.file_name().unwrap_or("upload.pdf").to_string();
            mime_type = field
                .content_type()
                .unwrap_or("application/pdf")
                .to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            file_data = data.to_vec();
            break;
        }
    }

    if file_data.is_empty() {
        return Err(AppError::BadRequest(
            "No file provided in 'file' field".into(),
        ));
    }

    let url = state
        .cloudinary_service
        .upload_file(file_data, &filename, &mime_type)
        .await?;

    let mut conn = state.pool.get()?;
    service::upload_license(&mut conn, user.id, &user, &url)?;

    Ok(ApiResponse::success_with_message(
        "License uploaded successfully",
        url,
    ))
}

/// Get patient profile for a specialist
///
/// Retrieves a patient's full profile including appointments and donation history.
#[utoipa::path(
    get,
    path = "/api/specialists/patients/{id}",
    responses(
        (status = 200, body = ApiResponse<crate::admin::dtos::AdminPatientProfileResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "User not found"),
        (status = 500)
    ),
    tag = "specialists",
    security(("bearer_auth" = []))
)]
pub async fn get_patient_profile(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(patient_id): Path<Uuid>,
) -> Result<ApiResponse<crate::admin::dtos::AdminPatientProfileResponse>, AppError> {
    use crate::schema::users;
    use crate::utils::enums::Role;

    if user.role != Role::Specialist {
        return Err(AppError::Unauthorized("Only specialists can access this".into()));
    }

    let mut conn = state.pool.get()?;

    // Fetch user to check that they are actually a patient
    let target_user = users::table
        .find(patient_id)
        .select(User::as_select())
        .first::<User>(&mut conn)
        .map_err(|_| AppError::NotFound("Patient not found".into()))?;

    match target_user.role {
        Role::Patient | Role::Donor | Role::PatientDonor => {
            let profile = crate::admin::dashboard::get_admin_patient_profile(&mut conn, patient_id)?;
            Ok(ApiResponse::success(profile))
        }
        _ => Err(AppError::BadRequest("User is not a patient".into()))
    }
}
