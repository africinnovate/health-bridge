use crate::error::AppError;
use crate::models::{EmailVerificationToken, NewEmailVerificationToken, NewUser, User};
use crate::schema::users;
use crate::utils::enums::Role;
use crate::utils::helpers::generate_numeric_code;
use anyhow::{Result, anyhow};
use argon2::{
    Argon2, 
    PasswordHash, 
    PasswordHasher, 
    PasswordVerifier,
    password_hash::{rand_core::OsRng, SaltString}
};
use argon2::password_hash::rand_core::RngCore;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, TokenData};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;
use chrono::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    exp: usize,
}

pub fn create_user(
    conn: &mut PgConnection,
    email: &str,
    password: &str,
    role: Role,
) -> Result<User> {

    // ---- Hash password using Argon2 ----
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("failed to hash password: {}", e))?
        .to_string();

    // ---- Prepare new user struct ----
    let new_user = NewUser {
        first_name: "",
        last_name: "",
        email,
        phone: None,
        gender: None,
        dob: None,
        password_hash: &password_hash,
        role,
    };

    // ---- Insert and return the newly created user ----
    let user = diesel::insert_into(users::table)
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
    
    let token = generate_reset_token();
    let expires_at = Utc::now() + Duration::hours(1);
    
    let new_token = crate::models::NewPasswordResetToken {
        user_id,
        token: &token,
        expires_at,
    };
    
    diesel::insert_into(password_reset_tokens::table)
        .values(&new_token)
        .execute(conn)?;
    
    Ok(token)
}

pub fn verify_email_code(
    conn: &mut PgConnection,
    user_email: &str,
    code: &str,
) -> Result<(), AppError> {
    use crate::schema::{users, email_verification_tokens};
    use diesel::prelude::*;
    use chrono::Utc;

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
    token: &str,
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
    token: &str,
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
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hashed = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|_| diesel::result::Error::RollbackTransaction)?
        .to_string();
 
    diesel::update(users.filter(id.eq(user_id)))
        .set(password_hash.eq(hashed))
        .execute(conn)?;
    
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
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|_| anyhow!("invalid credentials"))?;

    // ---- Verify password with Argon2 ----
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| anyhow!("invalid password hash"))?;

    let argon2 = Argon2::default();

    if argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok() {
        Ok(user)
    } else {
        Err(anyhow!("invalid credentials"))
    }
}


pub fn make_jwt(user_id: Uuid, secret: &str, expires_in_seconds: i64,) -> Result<String> {
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