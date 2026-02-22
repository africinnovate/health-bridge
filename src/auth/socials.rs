use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::config::SocialAuthConfig;
use crate::error::AppError;
use crate::handlers::socials::{GoogleTokenInfo, SocialProfile};
use crate::models::{NewSocialAccount, NewUser, SocialAccount, User};
use crate::utils::enums::Role;

pub fn login_or_register_social_user(
    conn: &mut PgConnection,
    profile: SocialProfile,
    provider: &str,
) -> Result<User, AppError> {
    use crate::schema::{social_accounts, users};

    // 1. Check if social account exists
    if let Ok(account) = social_accounts::table
        .filter(social_accounts::provider.eq(provider))
        .filter(social_accounts::provider_user_id.eq(&profile.provider_user_id))
        .first::<SocialAccount>(conn)
    {
        return users::table
            .find(account.user_id)
            .first::<User>(conn)
            .map_err(|_| AppError::NotFound("User not found".into()));
    }

    // 2. If email exists, link account
    let user = if let Some(email) = &profile.email {
        users::table
            .filter(users::email.eq(email))
            .first::<User>(conn)
            .optional()?
    } else {
        None
    };

    let user = match user {
        Some(u) => u,
        None => {
            // 3. Create new user
            let new_user = NewUser {
                first_name: profile.first_name.as_deref().unwrap_or(""),
                last_name: profile.last_name.as_deref().unwrap_or(""),
                email: profile.email.as_deref().unwrap_or(""),
                phone: None,
                gender: None,
                address: None,
                dob: None,
                image_url: None,
                password_hash: "", // IMPORTANT: no password
                role: Role::Patient,
                consultation_preference: None,
            };

            diesel::insert_into(users::table)
                .values(&new_user)
                .returning(User::as_returning())
                .get_result::<User>(conn)?
        }
    };

    // 4. Create social account link
    let link = NewSocialAccount {
        user_id: user.id,
        provider,
        provider_user_id: &profile.provider_user_id,
        email: profile.email.as_deref(),
    };

    diesel::insert_into(social_accounts::table)
        .values(&link)
        .execute(conn)?;

    Ok(user)
}

pub async fn verify_social_token(
    provider: &str,
    token: &str,
    cfg: &SocialAuthConfig,
) -> Result<SocialProfile, AppError> {
    match provider {
        "google" => verify_google_token(token, cfg).await,
        // "apple" => verify_apple_token(token, cfg).await,
        _ => Err(AppError::BadRequest("Unsupported provider".into())),
    }
}

pub async fn verify_google_token(
    token: &str,
    cfg: &SocialAuthConfig,
) -> Result<SocialProfile, AppError> {
    let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", token);

    let res = reqwest::get(&url).await?.json::<GoogleTokenInfo>().await?;

    if res.aud != cfg.google_client_id {
        return Err(AppError::Unauthorized("Invalid Google token".into()));
    }

    Ok(SocialProfile {
        provider_user_id: res.sub,
        email: res.email,
        first_name: res.given_name,
        last_name: res.family_name,
    })
}
