use axum::{Extension, Json, extract::{Path, Query, State}};
use uuid::Uuid;

use crate::{
    AppState,
    admin::{
        actions::{
            HospitalActionResponse, SpecialistActionResponse, update_hospital_status_with_audit,
            update_specialist_status_with_audit,
        },
        dashboard,
        dtos::{
            AdminDashboardResponse, HospitalActionRequest, SpecialistActionRequest, UserFilters,
            UserListResponse, AdminPatientProfileResponse, AdminUserProfileResponse,
        },
    },
    error::AppError,
    models::User,
    utils::{enums::Role, response::ApiResponse}
};

/// Get admin dashboard statistics
///
///  Retrieves key statistics for the admin dashboard
#[utoipa::path(
    get,
    path = "/api/admin/dashboard",
    responses(
        (status = 200, body = ApiResponse<AdminDashboardResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn admin_dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<AdminDashboardResponse>, AppError> {

    if user.role != Role::Admin {
        return Err(AppError::Unauthorized("Admin access required".into()));
    }

    let mut conn = state.pool.get()?;
    let dashboard = dashboard::get_admin_dashboard(&mut conn)?;

    Ok(ApiResponse::success(dashboard))
}


/// Get list of users
///
/// Retrieves a paginated list of users with optional filtering by role and search term
#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(UserFilters),
    responses(
        (status = 200, body = ApiResponse<UserListResponse>),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn get_users(
    State(state): State<AppState>,
    Query(query): Query<UserFilters>,
) -> Result<ApiResponse<UserListResponse>, AppError> {
    let mut conn = state.pool.get()?;

    // Validate page_size
    let page_size = query.page_size.min(100).max(1);
    let page = query.page.max(1);

    let response = dashboard::get_users_paginated(
        &mut conn,
        UserFilters {
            role: query.role,
            search: query.search,
            page,
            page_size,
        },
    )?;

    Ok(ApiResponse::success(response))
}



/// Update specialist verification/suspension status
///
/// Allows admins to verify, suspend, or unsuspend specialists
#[utoipa::path(
    patch,
    path = "/api/admin/specialists/{id}/status",
    request_body = SpecialistActionRequest,
    responses(
        (status = 200, body = ApiResponse<SpecialistActionResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Specialist not found"),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn update_specialist_status(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(specialist_id): Path<Uuid>,
    Json(payload): Json<SpecialistActionRequest>,
) -> Result<ApiResponse<SpecialistActionResponse>, AppError> {
    // Ensure user is admin
    if !matches!(user.role, crate::utils::enums::Role::Admin) {
        return Err(AppError::Unauthorized(
            "Only administrators can perform this action".into(),
        ));
    }

    // Validate: if suspending or removing verification, reason should be provided
    if let Some(false) = payload.verified {
        if payload.reason.is_none() || payload.reason.as_ref().unwrap().trim().is_empty() {
            return Err(AppError::BadRequest(
                "Reason is required when removing verification".into(),
            ));
        }
    }

    if let Some(true) = payload.suspended {
        if payload.reason.is_none() || payload.reason.as_ref().unwrap().trim().is_empty() {
            return Err(AppError::BadRequest(
                "Reason is required when suspending a specialist".into(),
            ));
        }
    }

    let mut conn = state.pool.get()?;

    let response = update_specialist_status_with_audit(
        &mut conn,
        specialist_id,
        &user,
        payload,
    )?;

    let action = if response.suspended {
        "suspended"
    } else if response.verified {
        "approved"
    } else {
        "updated"
    };

    Ok(ApiResponse::success_with_message(
        &format!("Specialist {} successfully", action),
        response,
    ))
}

/// Update hospital license status
///
/// Allows admins to approve or revoke hospital licenses
#[utoipa::path(
    patch,
    path = "/api/admin/hospitals/{id}/status",
    request_body = HospitalActionRequest,
    responses(
        (status = 200, body = ApiResponse<HospitalActionResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "Hospital not found"),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn update_hospital_status(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(hospital_id): Path<Uuid>,
    Json(payload): Json<HospitalActionRequest>,
) -> Result<ApiResponse<HospitalActionResponse>, AppError> {
    // Ensure user is admin
    if !matches!(user.role, crate::utils::enums::Role::Admin) {
        return Err(AppError::Unauthorized(
            "Only administrators can perform this action".into(),
        ));
    }

    // Validate: if revoking license, reason should be provided
    if let Some(false) = payload.license_status {
        if payload.reason.is_none() || payload.reason.as_ref().unwrap().trim().is_empty() {
            return Err(AppError::BadRequest(
                "Reason is required when revoking hospital license".into(),
            ));
        }
    }

    let mut conn = state.pool.get()?;

    let response = update_hospital_status_with_audit(
        &mut conn,
        hospital_id,
        &user,
        payload,
    )?;

    let action = if response.license_status {
        "approved"
    } else {
        "license revoked"
    };

    Ok(ApiResponse::success_with_message(
        &format!("Hospital {} successfully", action),
        response,
    ))
}

/// Get a specific user's full profile (Admin only)
///
/// Retrieves a comprehensive profile based on user role (patient, specialist, hospital)
#[utoipa::path(
    get,
    path = "/api/admin/users/{id}",
    responses(
        (status = 200, body = ApiResponse<AdminUserProfileResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin only"),
        (status = 404, description = "User not found"),
        (status = 500)
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn get_user_profile(
    State(state): State<AppState>,
    Extension(current_admin): Extension<User>,
    Path(user_id): Path<Uuid>,
) -> Result<ApiResponse<AdminUserProfileResponse>, AppError> {
    use crate::schema::users;
    use diesel::prelude::*;

    // Ensure user is admin
    if !matches!(current_admin.role, crate::utils::enums::Role::Admin) {
        return Err(AppError::Unauthorized(
            "Only administrators can perform this action".into(),
        ));
    }

    let mut conn = state.pool.get()?;
    
    // Fetch user to check role
    let target_user = users::table
        .find(user_id)
        .select(User::as_select())
        .first::<User>(&mut conn)
        .map_err(|_| AppError::NotFound("User not found".into()))?;

    let response = match target_user.role {
        Role::Patient | Role::Donor => {
            let profile = dashboard::get_admin_patient_profile(&mut conn, user_id)?;
            AdminUserProfileResponse::Patient(profile)
        }
        Role::Specialist => {
            let profile = dashboard::get_admin_specialist_profile(&mut conn, user_id)?;
            AdminUserProfileResponse::Specialist(profile)
        }
        Role::Hospital => {
            let profile = dashboard::get_admin_hospital_profile(&mut conn, user_id)?;
            AdminUserProfileResponse::Hospital(profile)
        }
        Role::Admin => {
            return Err(AppError::BadRequest("Cannot view admin profiles".into()));
        }
    };

    Ok(ApiResponse::success(response))
}
