use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    hospitals::{
        self,
        blood_requests::{
            BloodRequestQuery, BloodRequestResponse, CreateBloodRequest, UpdateBloodRequest,
        },
        settings,
    },
    models::{BloodRequest, HospitalSettings, User},
    utils::{
        enums::{BloodTypeEnum, HospitalTypeEnum, RequestStatusTypeEnum},
        response::{ApiResponse, EmptyData},
    },
};

/// Create hospital profile
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateHospitalRequest {
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub primary_phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub license_number: String,
    pub accreditation_doc_url: String,
    pub has_blood_bank: bool,
    pub accepting_donors: bool,
    pub donating_operating_hours: Option<String>,
    pub blood_inventory: Option<Vec<BloodInventoryUpdate>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BloodInventoryUpdate {
    pub blood_type: BloodTypeEnum,
    pub units_available: Option<i32>,
    pub bank_capacity: Option<i32>,
}

/// Update hospital profile (partial)
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateHospitalRequest {
    pub name: Option<String>,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub primary_phone: Option<String>,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub accreditation_doc_url: Option<String>,
    pub has_blood_bank: Option<bool>,
    pub accepting_donors: Option<bool>,
    pub donating_operating_hours: Option<String>,
    pub license_status: Option<bool>, // ADMIN ONLY
    pub blood_inventory: Option<Vec<BloodInventoryUpdate>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HospitalResponse {
    pub id: Uuid,
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub state: String,
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
    pub blood_inventory: Vec<crate::models::HospitalBloodInventory>,
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
) -> Result<ApiResponse<HospitalResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let hospital =
        hospitals::service::create_hospital(&mut conn, &user, payload, &state.mail_service)?;

    let response = HospitalResponse {
        id: hospital.id,
        name: hospital.name,
        hospital_type: hospital.hospital_type,
        address: hospital.address,
        city: hospital.city,
        state: hospital.state,
        country: hospital.country,
        primary_phone: hospital.primary_phone,
        emergency_phone: hospital.emergency_phone,
        email: hospital.email,
        license_number: hospital.license_number,
        accreditation_doc_url: hospital.accreditation_doc_url,
        license_status: hospital.license_status,
        has_blood_bank: hospital.has_blood_bank,
        accepting_donors: hospital.accepting_donors,
        donating_operating_hours: hospital.donating_operating_hours,
        created_at: hospital.created_at,
        blood_inventory: hospitals::service::get_hospital_inventory(&mut conn, hospital.id)?,
    };

    Ok(ApiResponse::success_with_message(
        "Hospital profile created successfully",
        response,
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
) -> Result<ApiResponse<HospitalResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let hospital = hospitals::service::update_hospital(&mut conn, hospital_id, &user, payload)?;
    let inventory = hospitals::service::get_hospital_inventory(&mut conn, hospital.id)?;

    let response = HospitalResponse {
        id: hospital.id,
        name: hospital.name,
        hospital_type: hospital.hospital_type,
        address: hospital.address,
        city: hospital.city,
        state: hospital.state,
        country: hospital.country,
        primary_phone: hospital.primary_phone,
        emergency_phone: hospital.emergency_phone,
        email: hospital.email,
        license_number: hospital.license_number,
        accreditation_doc_url: hospital.accreditation_doc_url,
        license_status: hospital.license_status,
        has_blood_bank: hospital.has_blood_bank,
        accepting_donors: hospital.accepting_donors,
        donating_operating_hours: hospital.donating_operating_hours,
        created_at: hospital.created_at,
        blood_inventory: inventory,
    };

    Ok(ApiResponse::success_with_message(
        "Hospital profile updated successfully",
        response,
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

    let request = hospitals::blood_requests::create_blood_request(&mut conn, &user, payload)?;

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
    Ok(ApiResponse::created(
        "Blood requests retrieved successfully",
        results,
    ))
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

    let request = hospitals::blood_requests::update_blood_request(&mut conn, id, &user, payload)?;

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
    Ok(ApiResponse::success_with_message(
        "Hospital settings retrieved successfully",
        settings,
    ))
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
    let settings = settings::update_hospital_settings(&mut conn, hospital_id, payload)?;
    Ok(ApiResponse::success_with_message(
        "Hospital settings updated successfully",
        settings,
    ))
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

/// Upload hospital accreditation document
///
/// The field name must be 'file' or 'accreditation_doc' and the content type must be 'multipart/form-data'.
#[utoipa::path(
    post,
    path = "/api/hospitals/upload-accreditation/{hospital_id}",
    params(
        ("hospital_id" = Uuid, Path, description = "Hospital ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = ApiResponse<String>)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn upload_accreditation_doc(
    State(state): State<AppState>,
    Path(hospital_id): Path<Uuid>,
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
        if name == "file" || name == "accreditation_doc" {
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
            "No file provided in 'file' or 'accreditation_doc' field".into(),
        ));
    }

    let url = state
        .cloudinary_service
        .upload_file(file_data, &filename, &mime_type)
        .await?;

    let mut conn = state.pool.get()?;
    hospitals::service::update_accreditation_doc(&mut conn, hospital_id, &user, &url)?;

    Ok(ApiResponse::success_with_message(
        "Accreditation document uploaded successfully",
        url,
    ))
}

/// Get all hospitals
#[utoipa::path(
    get,
    path = "/api/hospitals",
    responses(
        (status = 200, body = ApiResponse<Vec<HospitalResponse>>),
        (status = 500)
    ),
    tag = "hospitals"
)]
pub async fn get_hospitals(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<HospitalResponse>>, AppError> {
    let mut conn = state.pool.get()?;
    let hospitals = hospitals::service::get_hospitals(&mut conn)?;

    let mut response = Vec::new();
    for hospital in hospitals {
        let inventory = hospitals::service::get_hospital_inventory(&mut conn, hospital.id)?;
        response.push(HospitalResponse {
            id: hospital.id,
            name: hospital.name,
            hospital_type: hospital.hospital_type,
            address: hospital.address,
            city: hospital.city,
            state: hospital.state,
            country: hospital.country,
            primary_phone: hospital.primary_phone,
            emergency_phone: hospital.emergency_phone,
            email: hospital.email,
            license_number: hospital.license_number,
            accreditation_doc_url: hospital.accreditation_doc_url,
            license_status: hospital.license_status,
            has_blood_bank: hospital.has_blood_bank,
            accepting_donors: hospital.accepting_donors,
            donating_operating_hours: hospital.donating_operating_hours,
            created_at: hospital.created_at,
            blood_inventory: inventory,
        });
    }

    Ok(ApiResponse::success_with_message(
        "Hospitals retrieved successfully",
        response,
    ))
}

/// Get user's hospitals
#[utoipa::path(
    get,
    path = "/api/hospitals/me",
    responses(
        (status = 200, body = ApiResponse<Vec<HospitalResponse>>),
        (status = 401),
        (status = 500)
    ),
    tag = "hospitals",
    security(("bearer_auth" = []))
)]
pub async fn get_user_hospitals(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<Vec<HospitalResponse>>, AppError> {
    let mut conn = state.pool.get()?;
    let my_hospitals = hospitals::service::get_user_hospitals(&mut conn, user.id)?;

    let mut response = Vec::new();
    for hospital in my_hospitals {
        let inventory = hospitals::service::get_hospital_inventory(&mut conn, hospital.id)?;
        response.push(HospitalResponse {
            id: hospital.id,
            name: hospital.name,
            hospital_type: hospital.hospital_type,
            address: hospital.address,
            city: hospital.city,
            state: hospital.state,
            country: hospital.country,
            primary_phone: hospital.primary_phone,
            emergency_phone: hospital.emergency_phone,
            email: hospital.email,
            license_number: hospital.license_number,
            accreditation_doc_url: hospital.accreditation_doc_url,
            license_status: hospital.license_status,
            has_blood_bank: hospital.has_blood_bank,
            accepting_donors: hospital.accepting_donors,
            donating_operating_hours: hospital.donating_operating_hours,
            created_at: hospital.created_at,
            blood_inventory: inventory,
        });
    }

    Ok(ApiResponse::success_with_message(
        "User hospitals retrieved successfully",
        response,
    ))
}

/// Get hospital by ID
#[utoipa::path(
    get,
    path = "/api/hospitals/{hospital_id}",
    params(
        ("hospital_id" = Uuid, Path, description = "Hospital ID")
    ),
    responses(
        (status = 200, body = ApiResponse<HospitalResponse>),
        (status = 404),
        (status = 500)
    ),
    tag = "hospitals"
)]
pub async fn get_hospital_by_id(
    State(state): State<AppState>,
    Path(hospital_id): Path<Uuid>,
) -> Result<ApiResponse<HospitalResponse>, AppError> {
    let mut conn = state.pool.get()?;
    let hospital = hospitals::service::get_hospital_by_id(&mut conn, hospital_id)?;
    let inventory = hospitals::service::get_hospital_inventory(&mut conn, hospital.id)?;

    let response = HospitalResponse {
        id: hospital.id,
        name: hospital.name,
        hospital_type: hospital.hospital_type,
        address: hospital.address,
        city: hospital.city,
        state: hospital.state,
        country: hospital.country,
        primary_phone: hospital.primary_phone,
        emergency_phone: hospital.emergency_phone,
        email: hospital.email,
        license_number: hospital.license_number,
        accreditation_doc_url: hospital.accreditation_doc_url,
        license_status: hospital.license_status,
        has_blood_bank: hospital.has_blood_bank,
        accepting_donors: hospital.accepting_donors,
        donating_operating_hours: hospital.donating_operating_hours,
        created_at: hospital.created_at,
        blood_inventory: inventory,
    };

    Ok(ApiResponse::success_with_message(
        "Hospital retrieved successfully",
        response,
    ))
}
