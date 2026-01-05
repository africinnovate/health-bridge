use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
pub struct SocialAuthConfig {
    pub google_client_id: String,
}


#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in_seconds: i64,
    pub bind_addr: String,
    pub resend_api_key: String,
    pub from_email: String,
    pub frontend_url: String,
    pub social: SocialAuthConfig,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "change_me".into()),
            jwt_expires_in_seconds: env::var("JWT_EXPIRES_IN_SECONDS").unwrap_or_else(|_| "86400".into()).parse().expect("JWT_EXPIRES_IN_SECONDS must be a number"),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            resend_api_key: env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set"),
            from_email: env::var("FROM_EMAIL").unwrap_or_else(|_| "support@resend.dev".into()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".into()),
            social: SocialAuthConfig {
                google_client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            },
        }
    }
}
