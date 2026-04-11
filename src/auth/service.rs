use crate::error::AppError;
use crate::models::{
    EmailVerificationToken, NewEmailVerificationToken, NewRefreshToken, NewUser, RefreshToken, User,
};
use crate::schema::email_verification_tokens::dsl as evt;
use crate::schema::{password_reset_tokens, refresh_tokens};
use crate::utils::enums::Role;
use crate::utils::helpers::generate_numeric_code;
use anyhow::{Result, anyhow};
use argon2::password_hash::rand_core::RngCore;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use tracing::info;

use chrono::Duration;
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

pub fn create_user(
    conn: &mut PgConnection,
    user_email: &str,
    raw_password: &str,
    user_role: Role,
) -> Result<User> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    // Find soft-deleted user
    let deleted = users
        .filter(email.eq(user_email))
        .filter(deleted_at.is_not_null())
        .first::<User>(conn)
        .optional()?;

    // ---- Hash password ----
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed_password = argon2
        .hash_password(raw_password.as_bytes(), &salt)
        .map_err(|e| anyhow!("failed to hash password: {}", e))?
        .to_string();

    // ---- Restore soft-deleted user ----
    if let Some(deleted_user) = deleted {
        let restored = diesel::update(users.find(deleted_user.id))
            .set((
                password_hash.eq(&hashed_password),
                role.eq(user_role),
                deleted_at.eq::<Option<DateTime<Utc>>>(None), // ✅ FORCE NULL
            ))
            .returning(User::as_returning())
            .get_result::<User>(conn)?;

        return Ok(restored);
    }

    // ---- Insert new user ----
    // I'll use random bytes for an 8-character hex code.
    let random_code = format!("{:08X}", OsRng.next_u32());

    let new_user = NewUser {
        first_name: "",
        last_name: "",
        email: user_email,
        phone: None,
        gender: None,
        address: None,
        dob: None,
        image_url: None,
        password_hash: &hashed_password,
        role: user_role,
        consultation_preference: None,
        referral_code: &random_code,
        referral_link: None,
    };

    let user = diesel::insert_into(users)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result::<User>(conn)?;

    Ok(user)
}

pub fn create_email_verification_code(
    conn: &mut PgConnection,
    user_id: Uuid,
    length: usize,
) -> Result<String, diesel::result::Error> {
    use crate::schema::email_verification_tokens;

    let code = generate_numeric_code(length);
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

    let token = NewEmailVerificationToken {
        user_id,
        code: &code,
        expires_at,
    };

    diesel::insert_into(email_verification_tokens::table)
        .values(&token)
        .execute(conn)?;

    Ok(code)
}

pub fn resend_email_verification_code(
    conn: &mut PgConnection,
    user_email: &str,
) -> Result<String, AppError> {
    use crate::schema::{email_verification_tokens, users};
    use chrono::{Duration, Utc};
    use diesel::prelude::*;

    info!("Resending email verification code to {}", user_email);

    let user = users::table
        .filter(users::email.eq(user_email))
        .filter(users::deleted_at.is_null())
        .first::<User>(conn)
        .map_err(|_| AppError::BadRequest("User not found".into()))?;

    if user.email_verified {
        return Err(AppError::BadRequest("Email is already verified".into()));
    }

    let code = generate_numeric_code(4);
    let expires_at = Utc::now() + Duration::minutes(10);

    let new_token = NewEmailVerificationToken {
        user_id: user.id,
        code: &code,
        expires_at,
    };

    diesel::insert_into(email_verification_tokens::table)
        .values(&new_token)
        .on_conflict(email_verification_tokens::user_id)
        .do_update()
        .set((
            email_verification_tokens::code.eq(&code),
            email_verification_tokens::expires_at.eq(expires_at),
            email_verification_tokens::used.eq(false),
            email_verification_tokens::created_at.eq(Utc::now()),
        ))
        .execute(conn)?;

    Ok(code)
}

pub fn generate_reset_token() -> String {
    let mut rng = OsRng;
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn create_password_reset_token(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<String, diesel::result::Error> {
    use crate::schema::password_reset_tokens;

    let code = generate_numeric_code(6);
    let expires_at = Utc::now() + Duration::hours(1);

    let new_token = crate::models::NewPasswordResetToken {
        user_id,
        token: &code,
        expires_at,
    };

    diesel::insert_into(password_reset_tokens::table)
        .values(&new_token)
        .execute(conn)?;

    Ok(code)
}

pub fn verify_email_code(
    conn: &mut PgConnection,
    user_email: &str,
    code: &str,
) -> Result<(), AppError> {
    use crate::schema::{email_verification_tokens, users};
    use chrono::Utc;
    use diesel::prelude::*;

    let user = users::table
        .filter(users::email.eq(user_email))
        .first::<User>(conn)
        .map_err(|_| AppError::BadRequest("User not found".to_string()))?;

    let token = email_verification_tokens::table
        .filter(email_verification_tokens::user_id.eq(user.id))
        .filter(email_verification_tokens::code.eq(code))
        .filter(email_verification_tokens::used.eq(false))
        .filter(email_verification_tokens::expires_at.gt(Utc::now()))
        .first::<EmailVerificationToken>(conn)
        .map_err(|_| AppError::BadRequest("Invalid or expired verification code".to_string()))?;

    diesel::update(email_verification_tokens::table.find(token.id))
        .set(email_verification_tokens::used.eq(true))
        .execute(conn)?;

    diesel::update(users::table.find(user.id))
        .set(users::email_verified.eq(true))
        .execute(conn)?;

    Ok(())
}

pub fn verify_reset_token(
    conn: &mut PgConnection,
    _token: &str,
) -> Result<Uuid, diesel::result::Error> {
    use crate::schema::password_reset_tokens::dsl::*;
    use diesel::prelude::*;

    let reset_token = password_reset_tokens
        .filter(token.eq(token))
        .filter(used.eq(false))
        .filter(expires_at.gt(Utc::now()))
        .first::<crate::models::PasswordResetToken>(conn)?;

    Ok(reset_token.user_id)
}

pub fn mark_token_as_used(
    conn: &mut PgConnection,
    _token: &str,
) -> Result<(), diesel::result::Error> {
    use crate::schema::password_reset_tokens::dsl::*;
    use diesel::prelude::*;

    diesel::update(password_reset_tokens.filter(token.eq(token)))
        .set(used.eq(true))
        .execute(conn)?;

    Ok(())
}

pub fn reset_user_password(
    conn: &mut PgConnection,
    user_id: Uuid,
    new_password: &str,
) -> Result<(), AppError> {
    use crate::schema::refresh_tokens;
    use crate::schema::users;
    use diesel::prelude::*;

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::BadRequest(format!("Failed to hash password: {}", e)))?
        .to_string();

    conn.transaction::<_, AppError, _>(|conn| {
        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::password_hash.eq(hashed))
            .execute(conn)?;

        // Invalidate all refresh tokens
        diesel::update(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user_id)))
            .set(refresh_tokens::revoked.eq(true))
            .execute(conn)?;

        Ok(())
    })?;

    Ok(())
}

pub fn update_user_password(
    conn: &mut PgConnection,
    user_id: Uuid,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    use crate::schema::users;
    use diesel::prelude::*;

    // Fetch user
    let user = users::table
        .filter(users::id.eq(user_id))
        .filter(users::deleted_at.is_null())
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|_| AppError::NotFound("User not found".into()))?;

    // Verify old password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::BadRequest("Invalid password hash".into()))?;

    let argon2 = Argon2::default();
    if argon2
        .verify_password(old_password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::Unauthorized("Invalid old password".into()));
    }

    // Hash and update new password
    let salt = SaltString::generate(&mut OsRng);
    let hashed = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::BadRequest(format!("Failed to hash password: {}", e)))?
        .to_string();

    use crate::schema::refresh_tokens;

    conn.transaction::<_, AppError, _>(|conn| {
        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::password_hash.eq(hashed))
            .execute(conn)?;

        // Invalidate all refresh tokens
        diesel::update(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user_id)))
            .set(refresh_tokens::revoked.eq(true))
            .execute(conn)?;

        // User should be logged out

        Ok(())
    })?;

    Ok(())
}

pub fn authenticate_user(
    conn: &mut PgConnection,
    user_email: &str,
    password: &str,
) -> Result<User> {
    use crate::schema::users::dsl::*;

    // ---- Fetch user with Diesel ----
    let user = users
        .filter(email.eq(user_email))
        .filter(deleted_at.is_null())
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|_| anyhow!("invalid credentials"))?;

    // ---- Verify password with Argon2 ----
    let parsed_hash =
        PasswordHash::new(&user.password_hash).map_err(|_| anyhow!("invalid password hash"))?;

    let argon2 = Argon2::default();

    if argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        Ok(user)
    } else {
        Err(anyhow!("invalid credentials"))
    }
}

pub fn make_jwt(user_id: Uuid, secret: &str, expires_in_seconds: i64) -> Result<String> {
    use chrono::Utc;

    let exp = (Utc::now() + Duration::seconds(expires_in_seconds)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data)
}

pub fn soft_delete_account(conn: &mut PgConnection, user_id: &Uuid) -> Result<(), AppError> {
    use crate::schema::users::dsl::*;
    use crate::utils::enums::Role;
    use chrono::Utc;
    use diesel::prelude::*;

    let affected = diesel::update(
        users
            .filter(id.eq(user_id))
            .filter(role.ne(Role::Admin))
            .filter(deleted_at.is_null()),
    )
    .set(deleted_at.eq(Some(Utc::now())))
    .execute(conn)?;

    if affected == 0 {
        return Err(AppError::Unauthorized(
            "This account cannot be deleted".into(),
        ));
    }

    diesel::delete(evt::email_verification_tokens.filter(evt::user_id.eq(user_id)))
        .execute(conn)?;

    diesel::delete(password_reset_tokens::table.filter(password_reset_tokens::user_id.eq(user_id)))
        .execute(conn)?;

    diesel::delete(refresh_tokens::table.filter(refresh_tokens::user_id.eq(user_id)))
        .execute(conn)?;

    Ok(())
}

pub fn create_refresh_token(conn: &mut PgConnection, user_id: Uuid) -> Result<String, AppError> {
    use crate::schema::refresh_tokens;
    use chrono::{Duration, Utc};

    let token = generate_reset_token();
    let expires_at = Utc::now() + Duration::days(30);

    let new_token = NewRefreshToken {
        user_id,
        token: &token,
        expires_at,
    };

    diesel::insert_into(refresh_tokens::table)
        .values(new_token)
        .execute(conn)?;

    Ok(token)
}

pub fn refresh_access_token(
    conn: &mut PgConnection,
    refresh_token_value: &str,
    jwt_secret: &str,
    access_token_ttl: i64,
) -> Result<(String, String), AppError> {
    use crate::schema::refresh_tokens::dsl::*;
    use crate::schema::users::dsl as users_dsl;
    use chrono::Utc;

    // FIXED: Use refresh_token_value parameter to avoid shadowing
    let token_row = refresh_tokens
        .filter(token.eq(refresh_token_value))
        .filter(revoked.eq(false))
        .filter(expires_at.gt(Utc::now()))
        .select(RefreshToken::as_select())
        .first::<RefreshToken>(conn)
        .map_err(|e| {
            tracing::error!("Refresh token query failed: {:?}", e);
            AppError::Unauthorized("Invalid refresh token".into())
        })?;

    // Ensure user still exists and is not deleted
    let user = users_dsl::users
        .filter(users_dsl::id.eq(token_row.user_id))
        .filter(users_dsl::deleted_at.is_null())
        .first::<User>(conn)
        .map_err(|_| AppError::Unauthorized("User no longer active".into()))?;

    // Revoke old refresh token
    diesel::update(refresh_tokens.filter(id.eq(token_row.id)))
        .set(revoked.eq(true))
        .execute(conn)?;

    // Issue new tokens
    let access_token = make_jwt(user.id, jwt_secret, access_token_ttl)?;
    let new_refresh_token = create_refresh_token(conn, user.id)?;

    Ok((access_token, new_refresh_token))
}

/// Logout user by revoking their refresh token
pub fn logout_user(conn: &mut PgConnection, refresh_token_value: &str) -> Result<(), AppError> {
    use crate::schema::refresh_tokens::dsl::*;
    use diesel::prelude::*;

    let affected = diesel::update(refresh_tokens.filter(token.eq(refresh_token_value)))
        .set(revoked.eq(true))
        .execute(conn)?;

    if affected == 0 {
        return Err(AppError::BadRequest("Invalid refresh token".to_string()));
    }

    Ok(())
}

/// Logout user from all devices by revoking all their refresh tokens
pub fn logout_all_devices(conn: &mut PgConnection, user_id: Uuid) -> Result<(), AppError> {
    use crate::schema::refresh_tokens;
    use diesel::prelude::*;

    diesel::update(
        refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(user_id))
            .filter(refresh_tokens::revoked.eq(false)),
    )
    .set(refresh_tokens::revoked.eq(true))
    .execute(conn)?;

    Ok(())
}
