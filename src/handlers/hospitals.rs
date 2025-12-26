use axum::{
    Extension, Json, extract::{State, Path}
};
use diesel::AsChangeset;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    AppState, error::AppError, hospitals::{self, blood_requests::{CreateBloodRequest, UpdateBloodRequest}}, models::{BloodRequest, Hospital, User}, utils::{enums::HospitalTypeEnum, response::ApiResponse}
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

pub async fn create(
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

pub async fn update(
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

    Ok(ApiResponse::success(
        "Blood request updated successfully"
    ))
}