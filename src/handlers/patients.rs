use axum::{extract::State, Json, extract::Path};
use axum::Extension;
use serde::{Deserialize, Serialize};
use utoipa::openapi::info;
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::NaiveDate;
use tracing::{info};

use crate::models::{MedicalInfo, Patient, User};
use crate::schema::{users, patients};
use diesel::prelude::*;
use crate::{
    AppState,
    error::AppError,
    common,
    utils::{
        response::ApiResponse,
        validation::validate_phone_length,
        enums::Gender,
    }
};

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
}

#[derive(Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub role: String,
    pub email_verified: bool,
}

#[derive(Serialize, ToSchema)]
pub struct PatientProfileResponse {
    pub id: Uuid,

    // User fields
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub role: String,
    pub email_verified: bool,

    // Patient / medical fields
    pub blood_type: Option<String>,
    pub chronic_illnesses: Option<String>,
    pub allergies: Option<String>,
    pub hmo_number: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub medical_notes: Option<String>,
}


#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMedicalInfoRequest {
    pub blood_type: Option<String>,
    pub chronic_illnesses: Option<String>,
    pub allergies: Option<String>,
    pub hmo_number: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub medical_notes: Option<String>,
}


impl From<User> for ProfileResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            gender: user.gender,
            dob: user.dob,
            role: user.role.to_string(),
            email_verified: user.email_verified,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct DeleteAccountResponse {
    pub deleted: bool,
    pub user_id: Uuid,
}

/// Update user profile
#[utoipa::path(
    put,
    path = "/api/patients/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, body = ApiResponse<ProfileResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500)
    ),
    tag = "patients",
    security(("bearer_auth" = []))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<ApiResponse<ProfileResponse>, AppError> {
    if let Some(phone_number) = &payload.phone {
        validate_phone_length(phone_number, 11, 11)?;
    }

    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let updated_user = diesel::update(users.filter(id.eq(current_user.id)))
        .set((
            first_name.eq(&payload.first_name),
            last_name.eq(&payload.last_name),
            phone.eq(&payload.phone),
            gender.eq(&payload.gender),
            dob.eq(&payload.dob),
        ))
        .get_result::<User>(&mut conn)?;

    Ok(ApiResponse::success_with_message(
        "Profile updated successfully",
        updated_user.into(),
    ))
}


/// Get user profile
/// 
/// Returns the user's profile information, including medical details if available.
#[utoipa::path(
    get,
    path = "/api/patients/profile",
    responses(
        (status = 200, body = ApiResponse<PatientProfileResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "patients",
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Result<ApiResponse<PatientProfileResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let (user, patient) = users::table
        .left_join(patients::table.on(patients::user_id.eq(users::id)))
        .filter(users::id.eq(current_user.id))
        .select((User::as_select(), Option::<Patient>::as_select()))
        .first::<(User, Option<Patient>)>(&mut conn)?;

    let response = PatientProfileResponse {
        id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        phone: user.phone,
        gender: user.gender,
        dob: user.dob,
        role: user.role.to_string(),
        email_verified: user.email_verified,

        blood_type: patient.as_ref().and_then(|p| p.blood_type.clone()),
        chronic_illnesses: patient.as_ref().and_then(|p| p.chronic_illnesses.clone()),
        allergies: patient.as_ref().and_then(|p| p.allergies.clone()),
        hmo_number: patient.as_ref().and_then(|p| p.hmo_number.clone()),
        emergency_contact_name: patient.as_ref().and_then(|p| p.emergency_contact_name.clone()),
        emergency_contact_phone: patient.as_ref().and_then(|p| p.emergency_contact_phone.clone()),
        medical_notes: patient.as_ref().and_then(|p| p.medical_notes.clone()),
    };

    Ok(ApiResponse::success(response))
}



/// Delete user account
#[utoipa::path(
    delete,
    path = "/api/patients/profile",
    responses(
        (status = 200, body = ApiResponse<DeleteAccountResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "patients",
    security(("bearer_auth" = []))
)]
pub async fn delete_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Result<ApiResponse<DeleteAccountResponse>, AppError> {
    use diesel::prelude::*;

    let user_id = current_user.id;
    let mut conn = state.pool.get()?;

    conn.transaction::<_, AppError, _>(|conn| {
        use crate::schema::*;

        diesel::delete(password_reset_tokens::table.filter(password_reset_tokens::user_id.eq(user_id)))
            .execute(conn)?;

        diesel::delete(email_verification_tokens::table.filter(email_verification_tokens::user_id.eq(user_id)))
            .execute(conn)?;

        diesel::delete(patients::table.filter(patients::user_id.eq(user_id)))
            .execute(conn)
            .ok();

        let rows = diesel::delete(users::table.filter(users::id.eq(user_id)))
            .execute(conn)?;

        if rows == 0 {
            return Err(AppError::BadRequest("User not found".into()));
        }

        Ok(())
    })?;

    Ok(ApiResponse::success_with_message(
        "Account deleted successfully",
        DeleteAccountResponse {
            deleted: true,
            user_id,
        },
    ))
}

#[utoipa::path(
    put,
    path = "/api/patients/medical-info",
    request_body = UpdateMedicalInfoRequest,
    responses(
        (status = 200, description = "Medical info updated"),
        (status = 401),
        (status = 500)
    ),
    tag = "patients",
    security(("bearer_auth" = []))
)]
pub async fn update_medical_info(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<UpdateMedicalInfoRequest>,
) -> Result<ApiResponse<Patient>, AppError> {
    let mut conn = state.pool.get()?;

    let medical_info = MedicalInfo {
        user_id: current_user.id,
        blood_type: payload.blood_type.as_deref(),
        chronic_illnesses: payload.chronic_illnesses.as_deref(),
        allergies: payload.allergies.as_deref(),
        hmo_number: payload.hmo_number.as_deref(),
        emergency_contact_name: payload.emergency_contact_name.as_deref(),
        emergency_contact_phone: payload.emergency_contact_phone.as_deref(),
        medical_notes: payload.medical_notes.as_deref(),
    };

    let patient = common::services::upsert_medical_info(
        &mut conn,
        current_user.id,
        medical_info,
    )?;

    Ok(ApiResponse::success_with_message(
        "Medical information updated successfully",
        patient,
    ))
}
