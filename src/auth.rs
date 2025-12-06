use crate::models::{User, NewUser};
use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use diesel::prelude::*;
use diesel::pg::PgConnection;

use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, TokenData};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn create_user(
    conn: &mut PgConnection,
    email: &str,
    password: &str,
) -> Result<User> {

    // ---- Hash password using Argon2 ----
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &argon2::password_hash::SaltString::generate(&mut rand::thread_rng()))
        .map_err(|e| anyhow!("failed to hash password: {}", e))?
        .to_string();

    // ---- Prepare new user struct ----
    let new_user = NewUser {
        email,
        password_hash: &password_hash,
    };

    // ---- Insert and return the newly created user ----
    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(conn)?;

    Ok(user)
}


pub fn authenticate_user(
    conn: &mut PgConnection,
    email: &str,
    password: &str,
) -> Result<User> {

    use crate::schema::users::dsl::*;

    // ---- Fetch user with Diesel ----
    let user = users
        .filter(email.eq(email))
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


pub fn make_jwt(user_id: Uuid, secret: &str) -> Result<String> {
    use chrono::Utc;

    let exp = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;

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

