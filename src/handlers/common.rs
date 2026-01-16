use crate::{
    AppState,
    common::services,
    error::AppError,
    models::{User, UserSettings},
    utils::response::ApiResponse,
};
use axum::{
    Extension, Json,
    extract::{Multipart, State},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Upload user profile image
///
/// Uploads an image to Cloudinary and updates the user's profile with the new image URL.
#[utoipa::path(
    post,
    path = "/api/user-settings/upload-image",
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, body = ApiResponse<String>)
    ),
    tag = "settings",
    security(("bearer_auth" = []))
)]
pub async fn upload_image(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    mut multipart: Multipart,
) -> Result<ApiResponse<String>, AppError> {
    let mut image_data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "image" || name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            image_data = data.to_vec();
            break;
        }
    }

    if image_data.is_empty() {
        return Err(AppError::BadRequest(
            "No image file provided in 'image' or 'file' field".into(),
        ));
    }

    let image_url = state.cloudinary_service.upload_image(image_data).await?;

    let mut conn = state.pool.get()?;
    services::update_user_image(&mut conn, user.id, &image_url)?;

    Ok(ApiResponse::success(image_url))
}

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
