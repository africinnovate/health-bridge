use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    AppState, 
    auth::{socials, service as auth}, 
    error::AppError, 
    handlers::auth::AuthResponse, 
    utils::{response::ApiResponse, enums::Role},
};

#[derive(Deserialize)]
pub struct GoogleTokenInfo {
    pub sub: String,
    pub email: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub aud: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SocialLoginRequest {
    pub provider: String,
    pub access_token: String,
    pub role: Option<Role>,
}

#[derive(Serialize, ToSchema)]
pub struct SocialProfile {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Social login or register
///
/// Logs in or registers a user using social authentication providers like Google or Facebook.
#[utoipa::path(
    post,
    path = "/api/auth/social-login",
    request_body = SocialLoginRequest,
    responses(
        (status = 200, body = ApiResponse<AuthResponse>)
    ),
    tag = "auth"
)]
pub async fn social_login(
    State(state): State<AppState>,
    Json(payload): Json<SocialLoginRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let profile = socials::verify_social_token(
        &payload.provider,
        &payload.access_token,
        &state.cfg.social,
    )
    .await?;

    let mut conn = state.pool.get()?;

    let user = socials::login_or_register_social_user(
        &mut conn,
        profile,
        &payload.provider,
        payload.role,
    )?;

    let refresh_token = auth::create_refresh_token(&mut conn, user.id)?;
    let token = auth::make_jwt(
        user.id,
        &state.cfg.jwt_secret,
        state.cfg.jwt_expires_in_seconds,
    )?;

    Ok(ApiResponse::success(AuthResponse {
        token,
        refresh_token,
        user: user.into(),
    }))
}


