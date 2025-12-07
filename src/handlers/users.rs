use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info};
use utoipa::ToSchema;
use crate::{AppState, auth, models::User};

// Request/Response DTOs
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub role: String,
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
            role: user.role,
        }
    }
}

/// List all users
///
/// Gets all users on the application

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users retrieved successfully", body = [UserResponse]),
        (status = 500, description = "Internal server error")
    ),
    tag = "users"
)]
pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tracing::info!("after connectiondddd");
    let user_list = users
        .select(User::as_select())
        .load::<User>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<UserResponse> = user_list
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(response))
}

/// Register a new user
///
/// Creates a new user account and returns an authentication token
#[utoipa::path(
    post,
    path = "/api/users/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = AuthResponse),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let mut conn = state.pool.get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Check if email already exists
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    
    let existing_user = users
    .filter(email.eq(&payload.email))
    .first::<User>(&mut conn)
    .optional()
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("after connections");

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // Create user
    let user = auth::create_user(
        &mut conn,
        &payload.first_name,
        &payload.last_name,
        &payload.email,
        &payload.password,
        &payload.role,
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate JWT
    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// Login user
///
/// Authenticates a user and returns a JWT token
#[utoipa::path(
    post,
    path = "/api/users/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let mut conn = state.pool.get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Authenticate user
    let user = auth::authenticate_user(
        &mut conn,
        &payload.email,
        &payload.password,
    ).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Generate JWT
    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// Request password reset
///
/// Sends a password reset link to the user's email

#[utoipa::path(
    post,
    path = "/api/users/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Password reset email sent", body = MessageResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "authentication"
)]

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if user exists
    let user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if user.is_none() {
        // Don't reveal if email exists or not for security
        return Ok(Json(MessageResponse {
            message: "If the email exists, a password reset link has been sent.".to_string(),
        }));
    }

    // TODO: Generate reset token and send email
    // For now, just return success message
    // In production, you would:
    // 1. Generate a unique reset token
    // 2. Store it in a password_reset_tokens table with expiry
    // 3. Send email with reset link

    Ok(Json(MessageResponse {
        message: "If the email exists, a password reset link has been sent.".to_string(),
    }))
}

/// Reset password
///
/// Resets the user's password using a reset token
#[utoipa::path(
    post,
    path = "/api/users/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = MessageResponse),
        (status = 400, description = "Invalid or expired token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "authentication"
)]

pub async fn reset_password(
    State(_state): State<AppState>,
    Json(_payload): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, StatusCode> {
    // TODO: Implement password reset logic
    // 1. Verify reset token is valid and not expired
    // 2. Hash new password
    // 3. Update user's password
    // 4. Delete used reset token

    // Placeholder response
    Ok(Json(MessageResponse {
        message: "Password reset functionality not yet implemented.".to_string(),
    }))
}

/// Verify JWT token
///
/// Validates a JWT token and returns user information
#[utoipa::path(
    post,
    path = "/api/users/verify-token",
    request_body = VerifyTokenRequest,
    responses(
        (status = 200, description = "Token verification result", body = TokenVerifyResponse),
    ),
    tag = "authentication"
)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(payload): Json<VerifyTokenRequest>,
) -> Result<Json<TokenVerifyResponse>, StatusCode> {
    match auth::verify_jwt(&payload.token, &state.cfg.jwt_secret) {
        Ok(token_data) => Ok(Json(TokenVerifyResponse {
            valid: true,
            user_id: Some(token_data.claims.sub),
        })),
        Err(_) => Ok(Json(TokenVerifyResponse {
            valid: false,
            user_id: None,
        })),
    }
}