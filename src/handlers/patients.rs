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
    path = "/api/patients/update-profile/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ApiResponse<ProfileResponse>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "patients"
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<ApiResponse<ProfileResponse>, AppError> {
    
    if let Some(phone_number) = &payload.phone {
        validate_phone_length(phone_number, 11, 11)?;
    }

    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let updated_user = diesel::update(users.filter(id.eq(user_id)))
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
        ProfileResponse::from(updated_user),
    ))
}

/// Get user profile
#[utoipa::path(
    get,
    path = "/api/patients/get-profile/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ApiResponse<ProfileResponse>),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "patients"
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<ApiResponse<ProfileResponse>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user = users
        .filter(id.eq(user_id))
        .first::<User>(&mut conn)?;

    Ok(ApiResponse::success(ProfileResponse::from(user)))
}

/// Delete user account
#[utoipa::path(
    delete,
    path = "/api/patients/delete-account/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Account deleted successfully", body = ApiResponse<DeleteAccountResponse>),
        (status = 403, description = "Forbidden - Can only delete your own account"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "patients",
    security(
        ("bearer_auth" = [])
    )
)]

pub async fn delete_account(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Extension(current_user): Extension<User>,
) -> Result<ApiResponse<DeleteAccountResponse>, AppError> {
    use diesel::prelude::*;
    info!("User account about to be deleted3: {}", user_id);
    // Ensure the user can only delete their own account
    if current_user.id != user_id {
        return Err(AppError::Unauthorized);
    }

    let mut conn = state.pool.get()?;

    // Use a transaction to ensure all related data is deleted atomically
    conn.transaction::<_, AppError, _>(|conn| {
        // Delete password reset tokens
        {
            use crate::schema::password_reset_tokens::dsl::*;
            diesel::delete(password_reset_tokens.filter(user_id.eq(user_id)))
                .execute(conn)?;
        }

        // Delete email verification tokens
        {
            use crate::schema::email_verification_tokens::dsl::*;
            diesel::delete(email_verification_tokens.filter(user_id.eq(user_id)))
                .execute(conn)?;
        }

        // Delete patient record if exists
        {
            use crate::schema::patients::dsl::*;
            diesel::delete(patients.filter(user_id.eq(user_id)))
                .execute(conn)
                .ok();
        }

        // Finally, delete the user
        {
            use crate::schema::users::dsl::*;
            let rows_deleted = diesel::delete(users.filter(id.eq(user_id)))
                .execute(conn)?;

            if rows_deleted == 0 {
                return Err(AppError::BadRequest);
            }
        }

        info!("User account deleted: {}", user_id);

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