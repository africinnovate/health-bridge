use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::info;
use utoipa::ToSchema;

use crate::{
    AppState,
    auth,
    error::AppError,
    models::User,
};

// =====================
// DTOs
// =====================

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
) -> Result<Json<Vec<UserResponse>>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user_list = users
        .select(User::as_select())
        .load::<User>(&mut conn)?;

    Ok(Json(
        user_list.into_iter().map(UserResponse::from).collect(),
    ))
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
) -> Result<Json<AuthResponse>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let existing_user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    info!("after connections");

    if existing_user.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    let user = auth::create_user(
        &mut conn,
        &payload.first_name,
        &payload.last_name,
        &payload.email,
        &payload.password,
        &payload.role,
    )?;

    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)?;

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
) -> Result<Json<AuthResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let user = auth::authenticate_user(
        &mut conn,
        &payload.email,
        &payload.password,
    )
    .map_err(|_| AppError::Unauthorized)?;

    let token = auth::make_jwt(user.id, &state.cfg.jwt_secret)?;

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
) -> Result<Json<MessageResponse>, AppError> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let mut conn = state.pool.get()?;

    let user = users
        .filter(email.eq(&payload.email))
        .first::<User>(&mut conn)
        .optional()?;

    Ok(Json(MessageResponse {
        message: if user.is_some() {
            "If the email exists, a password reset link has been sent."
        } else {
            "If the email exists, a password reset link has been sent."
        }
        .to_string(),
    }))
}

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
) -> Result<Json<MessageResponse>, AppError> {
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
) -> Result<Json<TokenVerifyResponse>, AppError> {
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
