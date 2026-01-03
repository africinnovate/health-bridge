use axum::{Extension, extract::State};

use crate::{
  AppState, 
  admin::{dtos::AdminDashboardResponse, dashboard}, 
  error::AppError, 
  models::User, 
  utils::{enums::Role, response::ApiResponse}
};


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
