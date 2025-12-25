use axum::{extract::State, Json, extract::Path};
use axum::Extension;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::NaiveDate;
use tracing::{info};

use crate::{
    AppState,
    error::AppError,
    utils::{
        response::ApiResponse,
        validation::validate_phone_length,
        enums::Gender,
    },
    models::User,
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
#[utoipa::path(
    get,
    path = "/api/patients/profile",
    responses(
        (status = 200, body = ApiResponse<ProfileResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "patients",
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Result<ApiResponse<ProfileResponse>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user = users
        .filter(id.eq(current_user.id))
        .first::<User>(&mut conn)?;

    Ok(ApiResponse::success(user.into()))
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