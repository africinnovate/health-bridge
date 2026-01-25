use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    auth::service as auth,
    error::AppError,
    models::User,
    utils::{
        enums::Role,
        response::{ApiResponse, EmptyData},
        validation::validate_email,
    },
};

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub role: String,
    // pub phone: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub code: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct TokenVerifyResponse {
    pub valid: bool,
    pub user_id: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            role: user.role.to_string(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// Register a new user
///
/// Creates a new user account and returns an authentication token
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = ApiResponse<AuthResponse>),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    validate_email(&payload.email)?;
    // if let Some(phone_number) = &payload.phone {
    //     validate_phone_length(phone_number, 11, 11)?;
    // }
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = app_state.pool.get()?;

    let existing_user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    let other_role = payload.role.parse::<Role>()?;

    if let Some(user) = existing_user {
        if user.deleted_at.is_none() {
            return Err(AppError::UserAlreadyExists);
        }
    }

    let user = auth::create_user(&mut conn, &payload.email, &payload.password, other_role)?;

    let code = auth::create_email_verification_code(&mut conn, user.id, 4)?;

    let mail = app_state.mail_service.clone();
    let other_email = user.email.clone();

    tokio::spawn(async move {
        let _ = mail.send_notification(
            &other_email,
            "Verify your HealthBridge account",
            &format!(
                "<p>Your verification code is:</p><h2>{}</h2><p>This code expires in 10 minutes.</p>",
                code
            ),
            Some(&format!("Your verification code is: {}", code)),
        ).await;
    });

    Ok(ApiResponse::created(
        "User registered successfully",
        AuthResponse {
            token: "".to_string(),
            refresh_token: "".to_string(),
            user: user.into(),
        },
    ))
}

/// Login user
///
/// Authenticates a user and returns a JWT token
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<AuthResponse>),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    validate_email(&payload.email)?;
    let mut conn = state.pool.get()?;

    let user = auth::authenticate_user(&mut conn, &payload.email, &payload.password)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let refresh_token = auth::create_refresh_token(&mut conn, user.id)?;

    let token = auth::make_jwt(
        user.id,
        &state.cfg.jwt_secret,
        state.cfg.jwt_expires_in_seconds,
    )?;

    Ok(ApiResponse::success_with_message(
        "Login successful",
        AuthResponse {
            token,
            refresh_token,
            user: user.into(),
        },
    ))
}

/// Request password reset
///
/// Sends a password reset link to the user's email

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = ApiResponse<EmptyData>),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]

pub async fn forgot_password(
    State(app_state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    validate_email(&payload.email)?;

    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = app_state.pool.get()?;

    let user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    if let Some(user) = user {
        // Generate reset token
        let reset_code = auth::create_password_reset_token(&mut conn, user.id)?;

        // Send email asynchronously
        let mail_service = app_state.mail_service.clone();
        let user_email = user.email.clone();
        let user_name = format!("{} {}", user.first_name, user.last_name);

        tokio::spawn(async move {
            if let Err(e) = mail_service
                .send_password_reset_email(&user_email, &user_name, &reset_code)
                .await
            {
                error!("Failed to send password reset email: {:?}", e);
            }
        });

        info!("Password reset code generated for user: {}", user.email);
    }

    // Always return success to prevent email enumeration
    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "If the email exists, a password reset link has been sent.",
    ))
}

/// Reset password
///
/// Endpoint to reset password for users who have requested a password reset
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid or expired token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    // Verify the token and get user_id
    let user_id = auth::verify_reset_token(&mut conn, &payload.code)
        .map_err(|_| AppError::BadRequest("Invalid or expired code".to_string()))?;

    // Reset the password
    auth::reset_user_password(&mut conn, user_id, &payload.new_password)?;

    // Mark token as used
    auth::mark_token_as_used(&mut conn, &payload.code)?;

    info!("Password successfully reset for user: {}", user_id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been reset successfully.",
    ))
}

/// Update password
///
/// Updates the password for the authenticated user
#[utoipa::path(
    post,
    path = "/api/auth/update-password",
    request_body = UpdatePasswordRequest,
    responses(
        (status = 200, description = "Password updated successfully", body = ApiResponse<EmptyData>),
        (status = 401, description = "Invalid old password"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn update_password(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    auth::update_user_password(
        &mut conn,
        user.id,
        &payload.old_password,
        &payload.new_password,
    )?;

    info!("Password updated for user: {}", user.id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been updated successfully.",
    ))
}

/// Verify JWT token
///
/// Validates a JWT token and returns user information
#[utoipa::path(
    post,
    path = "/api/auth/verify-token",
    request_body = VerifyTokenRequest,
    responses(
        (status = 200, description = "Token verification result", body = ApiResponse<TokenVerifyResponse>),
    ),
    tag = "auth"
)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<ApiResponse<TokenVerifyResponse>, AppError> {
    let response = match auth::verify_jwt(&payload.token, &state.cfg.jwt_secret) {
        Ok(token_data) => TokenVerifyResponse {
            valid: true,
            user_id: Some(token_data.claims.sub),
        },
        Err(_) => TokenVerifyResponse {
            valid: false,
            user_id: None,
        },
    };

    Ok(ApiResponse::success(response))
}

/// Verify email verification code
///
/// Validates the email verification code sent to the user's email

#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid or expired verification code"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn verify_email(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    validate_email(&payload.email)?;

    let mut conn = state.pool.get()?;

    auth::verify_email_code(&mut conn, &payload.email, &payload.code)?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Email verified successfully.",
    ))
}

/// Resend email verification code
///
/// Sends a new verification code to the user's email

#[utoipa::path(
    post,
    path = "/api/auth/resend-verification-code",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification code resent", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn resend_verification_code(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    info!("Resending verification code to {}", &payload.email);
    validate_email(&payload.email)?;

    let mut conn = state.pool.get()?;

    let code = auth::resend_email_verification_code(&mut conn, &payload.email)?;

    let mail_service = state.mail_service.clone();
    let email = payload.email.clone();

    tokio::spawn(async move {
        let _ = mail_service.send_notification(
            &email,
            "Verify your HealthBridge account",
            &format!(
                "<p>Your new verification code is:</p><h2>{}</h2><p>This code expires in 10 minutes.</p>",
                code
            ),
            Some(&format!("Your verification code is: {}", code)),
        ).await;
    });

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Verification code sent successfully.",
    ))
}

/// Delete user account
///
/// Permanently deletes the authenticated user's account
#[utoipa::path(
    post,
    path = "/api/auth/delete-account",
    responses(
        (status = 200, description = "Account deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn delete_account(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;
    info!("Deleting account for user: {}", &user.id);
    auth::soft_delete_account(&mut conn, &user.id)?;

    let mail_service = state.mail_service.clone();
    let user_email = user.email.clone();
    let user_name = format!("{} {}", user.first_name, user.last_name);

    // Send email asynchronously
    tokio::spawn(async move {
        let subject = "Your HealthBridge account has been deleted";
        let html_body = format!(
            r#"
            <p>Hi {},</p>
            <p>Your HealthBridge account has been successfully deleted.</p>
            <p>If this action was not initiated by you, please contact our support team immediately.</p>
            <p>— HealthBridge Team</p>
            "#,
            user_name
        );

        let text_body = format!(
            "Hi {},\n\nYour HealthBridge account has been successfully deleted.\nIf this wasn’t you, please contact support.\n\n— HealthBridge Team",
            user_name
        );

        if let Err(e) = mail_service
            .send_notification(&user_email, subject, &html_body, Some(&text_body))
            .await
        {
            error!("Failed to send account deletion email: {:?}", e);
        }
    });

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Account deleted successfully",
    ))
}

/// Refresh access token
///
/// Generates a new access token using a valid refresh token

#[utoipa::path(
    post,
    path = "/api/auth/refresh-token",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed", body = ApiResponse<RefreshTokenResponse>),
        (status = 401, description = "Invalid refresh token")
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<ApiResponse<RefreshTokenResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let (access_token, refresh_token) = auth::refresh_access_token(
        &mut conn,
        &payload.refresh_token,
        &state.cfg.jwt_secret,
        state.cfg.jwt_expires_in_seconds,
    )?;

    Ok(ApiResponse::success(RefreshTokenResponse {
        access_token,
        refresh_token,
    }))
}

/// Logout user
///
/// Revokes the provided refresh token, logging out the user from the current device

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logout successful", body = ApiResponse<EmptyData>),
        (status = 400, description = "Invalid refresh token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<LogoutRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    auth::logout_user(&mut conn, &payload.refresh_token)?;

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Logout successful",
    ))
}

/// Logout from all devices
///
/// Revokes all refresh tokens for the authenticated user, logging them out from all devices

#[utoipa::path(
    post,
    path = "/api/auth/logout-all",
    responses(
        (status = 200, description = "Logged out from all devices", body = ApiResponse<EmptyData>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth",
    security(("bearer_auth" = []))
)]
pub async fn logout_all_devices(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<EmptyData>, AppError> {
    let mut conn = state.pool.get()?;

    auth::logout_all_devices(&mut conn, user.id)?;

    info!("User {} logged out from all devices", user.id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Logged out from all devices successfully",
    ))
}
