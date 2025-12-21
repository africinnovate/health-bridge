use axum::{extract::State, Json, extract::Path};
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
    tag = "profile"
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
    tag = "profile"
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