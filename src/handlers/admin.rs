use axum::{Extension, extract::{Query, State}};

use crate::{
  AppState, 
  admin::{dashboard, dtos::{AdminDashboardResponse, UserFilters, UserListResponse}}, 
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