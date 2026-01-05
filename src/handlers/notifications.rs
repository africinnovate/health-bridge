use axum::{Extension, extract::{Path, Query, State}};
use uuid::Uuid;

use crate::{
  AppState, admin::{
    dtos::{ NotificationFilters}
}, 
    common::notifications, 
    error::AppError, 
    models::User, 
    utils::{response::ApiResponse}
};



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

