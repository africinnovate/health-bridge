use axum::{extract::State, Json, http::StatusCode,};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error };
use utoipa::ToSchema;

use crate::{
    AppState,
    auth,
    error::AppError,
    models::User,
    utils::{
        enums::Role, response::{ApiResponse, EmptyData}, validation::{validate_email, validate_phone_length}
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
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
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
pub struct MessageResponse {
    pub message: String,
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

/// List all users
///
/// Gets all users on the application

#[utoipa::path(
    get,
    path = "/api/auth",
    responses(
        (status = 200, description = "List of users retrieved successfully", body = ApiResponse<Vec<UserResponse>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn get_users(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<UserResponse>>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user_list = users
        .select(User::as_select())
        .load::<User>(&mut conn)?;

    let users_response: Vec<UserResponse> = user_list
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(ApiResponse::success_with_message(
        "Users retrieved successfully",
        users_response,
    ))
}

/// Register a new user
///
/// Creates a new user account and returns an authentication token
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = ApiResponse<AuthResponse>),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {

    validate_email(&payload.email)?;
    // if let Some(phone_number) = &payload.phone {
    //     validate_phone_length(phone_number, 11, 11)?;
    // }
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let existing_user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    if existing_user.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    let other_role = payload
    .role
    .parse::<Role>()?;

    let user = auth::create_user(
        &mut conn,
        &payload.email,
        &payload.password,
        other_role,
    )?;

    let code = auth::create_email_verification_code(&mut conn, user.id, 4)?;
    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)?;

    let mail = state.mail_service.clone();
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
            token,
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

    let user = auth::authenticate_user(
        &mut conn,
        &payload.email,
        &payload.password,
    )
    .map_err(|_| AppError::Unauthorized)?;

    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)?;

    Ok(ApiResponse::success_with_message(
        "Login successful",
        AuthResponse {
            token,
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
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<ApiResponse<EmptyData>, AppError> {

    validate_email(&payload.email)?;
    
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    if let Some(user) = user {
        // Generate reset token
        let reset_token = auth::create_password_reset_token(&mut conn, user.id)?;
        
        // Send email asynchronously
        let mail_service = state.mail_service.clone();
        let user_email = user.email.clone();
        let user_name = format!("{} {}", user.first_name, user.last_name);
        let frontend_url = state.cfg.frontend_url.clone();
        
        tokio::spawn(async move {
            if let Err(e) = mail_service
                .send_password_reset_email(&user_email, &user_name, &reset_token, &frontend_url)
                .await
            {
                error!("Failed to send password reset email: {:?}", e);
            }
        });
        
        info!("Password reset token generated for user: {}", user.email);
    }

    // Always return success to prevent email enumeration
    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "If the email exists, a password reset link has been sent.",
    ))
}

/// Reset password
///
/// Endpoint to reset password
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
    let user_id = auth::verify_reset_token(&mut conn, &payload.token)
        .map_err(|_| AppError::BadRequest)?;

    // Reset the password
    auth::reset_user_password(&mut conn, user_id, &payload.new_password)?;

    // Mark token as used
    auth::mark_token_as_used(&mut conn, &payload.token)?;

    info!("Password successfully reset for user: {}", user_id);

    Ok(ApiResponse::message_only(
        StatusCode::OK,
        "Password has been reset successfully.",
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
