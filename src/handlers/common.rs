use axum::{extract::State, Json, Extension};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::{
    AppState, 
    common::services, 
    error::AppError, 
    models::{User, UserSettings},
    utils::response::ApiResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserSettingsRequest {
    pub appointment_reminders: Option<bool>,
    pub specialist_recommendations: Option<bool>,
    pub donation_alerts: Option<bool>,
    pub account_notifications: Option<bool>,

    pub email_notifications: Option<bool>,
    pub sms_notifications: Option<bool>,
    pub push_notifications: Option<bool>,

    pub medical_profile_visibility: Option<String>,
    pub allow_specialists_view_history: Option<bool>,
    pub allow_app_analytics: Option<bool>,
    pub allow_marketing_notifications: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserSettingsResponse {
    pub appointment_reminders: bool,
    pub specialist_recommendations: bool,
    pub donation_alerts: bool,
    pub account_notifications: bool,

    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub push_notifications: bool,

    pub medical_profile_visibility: String,
    pub allow_specialists_view_history: bool,
    pub allow_app_analytics: bool,
    pub allow_marketing_notifications: bool,
}

/// Get user settings
/// 
/// Retrieves the notification and privacy settings for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/user-settings",
    responses(
        (status = 200, body = ApiResponse<UserSettingsResponse>)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn get_user_settings(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<UserSettingsResponse>, AppError> {
    let mut conn = state.pool.get()?;
    let settings = services::get_or_create_user_settings(&mut conn, &user)?;

    Ok(ApiResponse::success(settings.into()))
}

/// Update user settings
/// 
/// Updates the notification and privacy settings for the authenticated user.
#[utoipa::path(
    put,
    path = "/api/user-settings",
    request_body = UpdateUserSettingsRequest,
    responses(
        (status = 200, body = ApiResponse<UserSettingsResponse>)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn update_user_settings(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateUserSettingsRequest>,
) -> Result<ApiResponse<UserSettingsResponse>, AppError> {
    let mut conn = state.pool.get()?;
    let settings = services::update_user_settings(&mut conn, &user, payload)?;

    Ok(ApiResponse::success(settings.into()))
}
impl From<UserSettings> for UserSettingsResponse {
    fn from(settings: UserSettings) -> Self {
        Self {
            appointment_reminders: settings.appointment_reminders,
            specialist_recommendations: settings.specialist_recommendations,
            donation_alerts: settings.donation_alerts,
            account_notifications: settings.account_notifications,
            email_notifications: settings.email_notifications,
            sms_notifications: settings.sms_notifications,
            push_notifications: settings.push_notifications,
            medical_profile_visibility: settings.medical_profile_visibility,
            allow_specialists_view_history: settings.allow_specialists_view_history,
            allow_app_analytics: settings.allow_app_analytics,
            allow_marketing_notifications: settings.allow_marketing_notifications,
        }
    }
}