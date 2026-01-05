use axum::{Extension, Json, extract::{Path, Query, State}};
use uuid::Uuid;

use crate::{
  AppState, admin::{
    actions::{HospitalActionResponse, SpecialistActionResponse, update_hospital_status_with_audit, update_specialist_status_with_audit}, 
    dashboard, dtos::{AdminDashboardResponse, HospitalActionRequest, NotificationFilters, SpecialistActionRequest, UserFilters, UserListResponse}
}, 
    common::notifications, 
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


/// Get paginated notifications
/// 
/// Retrieves notifications for the current user. Admins can see all notifications and filter by category.
#[utoipa::path(
    get,
    path = "/api/notifications",
    params(NotificationFilters),
    responses(
        (status = 200, body = ApiResponse<notifications::NotificationListResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500)
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn get_notifications(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(query): Query<NotificationFilters>,
) -> Result<ApiResponse<notifications::NotificationListResponse>, AppError> {
    let mut conn = state.pool.get()?;

    // Validate page_size
    let page_size = query.page_size.min(100).max(1);
    let page = query.page.max(1);

    let response = notifications::get_notifications_paginated(
        &mut conn,
        &user,
        NotificationFilters {
            category: query.category,
            is_read: query.is_read,
            search: query.search,
            page,
            page_size,
        },
    )?;

    Ok(ApiResponse::success(response))
}

/// Mark notification as read
/// 
/// Marks a specific notification as read
#[utoipa::path(
    patch,
    path = "/api/notifications/{id}/read",
    responses(
        (status = 200, body = ApiResponse<notifications::NotificationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not your notification"),
        (status = 404, description = "Notification not found"),
        (status = 500)
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn mark_notification_as_read(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(notification_id): Path<Uuid>,
) -> Result<ApiResponse<notifications::NotificationResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let response = notifications::mark_notification_as_read(&mut conn, notification_id, &user)?;

    Ok(ApiResponse::success_with_message(
        "Notification marked as read",
        response,
    ))
}

/// Mark all notifications as read
/// 
/// Marks all unread notifications for the current user as read
#[utoipa::path(
    patch,
    path = "/api/notifications/read-all",
    responses(
        (status = 200, body = ApiResponse<notifications::MarkAsReadResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 500)
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn mark_all_notifications_as_read(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<notifications::MarkAsReadResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let response = notifications::mark_all_notifications_as_read(&mut conn, &user)?;

    Ok(ApiResponse::success_with_message(
        &format!("{} notifications marked as read", response.marked_count),
        response,
    ))
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}