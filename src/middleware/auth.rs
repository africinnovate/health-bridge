use axum::{
    Extension, extract::{Request, State}, http::{StatusCode, header}, middleware::Next, response::Response
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use tracing::{info};

use crate::{
    AppState,
    error::AppError,
    models::User,
    schema::users,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,
}

pub async fn require_auth(
    Extension(state): Extension<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing or invalid Authorization header".to_string()))?;

    // Extract bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Missing or invalid Bearer token".to_string()))?;

    // Decode and validate token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    // Parse user_id from claims
    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;
    // Fetch user from database
    let mut conn = state.pool.get()?;
    let user = users::table
        .filter(users::id.eq(user_id))
        .first::<User>(&mut conn)
        .map_err(|_| AppError::Unauthorized("User not found".to_string()))?;

    // Insert user into request extensions
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}