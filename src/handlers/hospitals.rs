use axum::{
    Extension, Json, extract::{Path, Query, State}
};
use diesel::AsChangeset;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    AppState, 
    error::AppError, 
    hospitals::{
        self, 
        blood_requests::{BloodRequestQuery, BloodRequestResponse, CreateBloodRequest, UpdateBloodRequest}, settings}, 
        models::{BloodRequest, Hospital, HospitalSettings, User}, 
        utils::{
            enums::{HospitalTypeEnum, RequestStatusTypeEnum}, 
            response::{ApiResponse, EmptyData}
        },
};

/// Create hospital profile
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHospitalRequest {
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub country: String,
    pub primary_phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub license_number: String,
    pub accreditation_doc_url: String,
    pub has_blood_bank: bool,
    pub accepting_donors: bool,
    pub donating_operating_hours: Option<String>,
}

/// Update hospital profile (partial)
#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = crate::schema::hospitals)]

pub struct UpdateHospitalRequest {
    pub name: Option<String>,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub primary_phone: Option<String>,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub accreditation_doc_url: Option<String>,
    pub has_blood_bank: Option<bool>,
    pub accepting_donors: Option<bool>,
    pub donating_operating_hours: Option<String>,
    pub license_status: Option<bool>, // ADMIN ONLY
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HospitalResponse {
    pub id: Uuid,
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub country: String,
    pub primary_phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub license_number: String,
    pub accreditation_doc_url: String,
    pub license_status: bool,
    pub has_blood_bank: bool,
    pub accepting_donors: bool,
    pub donating_operating_hours: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateHospitalSettingsRequest {
    pub donation_requests: Option<bool>,

    pub new_donor_appointments: Option<bool>,
    pub donor_appointment_reminders: Option<bool>,

    pub login_alerts: Option<bool>,
    pub account_notifications: Option<bool>,

    pub email_notifications: Option<bool>,
    pub sms_notifications: Option<bool>,
    pub push_notifications: Option<bool>,
}



/// Create hospital
#[utoipa::path(
    post,
    path = "/api/hospitals/create",
    request_body = CreateHospitalRequest,
    responses(
        (status = 200, body = ApiResponse<HospitalResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn create_hospital(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateHospitalRequest>,
) -> Result<ApiResponse<Hospital>, AppError> {
    let mut conn = state.pool.get()?;

    let hospital = hospitals::service::create_hospital(
        &mut conn,
        &user,
        payload,
        &state.mail_service,
    )?;

    Ok(ApiResponse::success_with_message(
        "Hospital profile created successfully",
        hospital,
    ))
}

/// Update hospital
#[utoipa::path(
    put,
    path = "/api/hospitals/update/{hospital_id}",
    request_body = UpdateHospitalRequest,
    responses(
        (status = 200, body = ApiResponse<HospitalResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]

pub async fn update_hospital(
    State(state): State<AppState>,
    Path(hospital_id): Path<Uuid>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateHospitalRequest>,
) -> Result<ApiResponse<Hospital>, AppError> {
    let mut conn = state.pool.get()?;

    let hospital = hospitals::service::update_hospital(
        &mut conn,
        hospital_id,
        &user,
        payload,
    )?;

    Ok(ApiResponse::success_with_message(
        "Hospital profile updated successfully",
        hospital,
    ))
}
/// Create blood request
#[utoipa::path(
    post,
    path = "/api/hospitals/blood-request",
    request_body = CreateBloodRequest,
    responses(
        (status = 200, body = ApiResponse<BloodRequest>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]


pub async fn create_blood_request(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateBloodRequest>,
) -> Result<ApiResponse<BloodRequest>, AppError> {
    let mut conn = state.pool.get()?;

    let request = hospitals::blood_requests::create_blood_request(
        &mut conn,
        &user,
        payload,
    )?;

    Ok(ApiResponse::created(
        "Blood request created successfully",
        request,
    ))
}

/// Get blood requests for hospital
///     
/// Retrieves blood requests associated with the hospital of the authenticated user.
#[utoipa::path(
    get,
    path = "/api/hospitals/blood-request",
    params(
        ("request_status" = Option<RequestStatusTypeEnum>, Query)
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<BloodRequest>>),
        (status = 401)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn get_blood_requests(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(filters): Query<BloodRequestQuery>,
) -> Result<ApiResponse<Vec<BloodRequestResponse>>, AppError> {
    let mut conn = state.pool.get()?;
    let results = hospitals::blood_requests::get_blood_requests(&mut conn, &user, filters)?;
    Ok(ApiResponse::created("Blood requests retrieved successfully", results))
}


/// Update blood request
#[utoipa::path(
    put,
    path = "/api/hospitals/blood-request/{hospital_id}",
    request_body = UpdateBloodRequest,
    responses(
        (status = 200, body = ApiResponse<BloodRequest>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]

pub async fn update_blood_request(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBloodRequest>,
) -> Result<ApiResponse<BloodRequest>, AppError> {
    let mut conn = state.pool.get()?;

    let request = hospitals::blood_requests::update_blood_request(
        &mut conn,
        id,
        &user,
        payload,
    )?;

    Ok(ApiResponse::success_with_message(
        "Blood request updated successfully",
        request,
    ))  
}

/// Get hospital settings
/// 
/// Retrieves settings for the specified hospital.

#[utoipa::path(
    get,
    path = "/api/hospitals/settings/{hospital_id}",
    responses(
        (status = 200, body = ApiResponse<HospitalSettings>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn get_hospital_settings(
    State(state): State<AppState>,
    Path(hospital_id): Path<Uuid>,
) -> Result<ApiResponse<HospitalSettings>, AppError> {
    let mut conn = state.pool.get()?;
    let settings = settings::get_or_create_hospital_settings(&mut conn, hospital_id)?;
    Ok(ApiResponse::success_with_message("Hospital settings retrieved successfully", settings))
}

/// Update hospital settings
///
/// Updates the settings for the specified hospital.
#[utoipa::path(
    patch,
    path = "/api/hospitals/settings/{hospital_id}",
    request_body = UpdateHospitalSettingsRequest,
    responses(
        (status = 200, body = ApiResponse<HospitalSettings>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn update_hospital_settings(
    State(state): State<AppState>,
    Path(hospital_id): Path<Uuid>,
    Json(payload): Json<UpdateHospitalSettingsRequest>,
) -> Result<ApiResponse<HospitalSettings>, AppError> {
    let mut conn = state.pool.get()?;
    let settings =
        settings::update_hospital_settings(&mut conn, hospital_id, payload)?;
    Ok(ApiResponse::success_with_message("Hospital settings updated successfully", settings))
}


/// Delete hospital
#[utoipa::path(
    delete,
    path = "/api/hospitals/{hospital_id}",
    responses(
        (status = 200, description = "Hospital deleted successfully"),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn delete_hospital(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(hospital_id): Path<Uuid>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    hospitals::service::delete_hospital(&mut conn, hospital_id, &user)?;

    Ok(ApiResponse::message_only(
        axum::http::StatusCode::OK,
        "Hospital deleted successfully",
    ))
}

