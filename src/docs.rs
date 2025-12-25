use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};
use crate::handlers::auth;
use crate::handlers::hospitals;
use crate::handlers::patients;
use crate::handlers::specialists;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::get_users,
        auth::register,
        auth::login,
        auth::forgot_password,
        auth::reset_password,
        auth::verify_token,
        auth::verify_email,
        patients::get_profile,
        patients::update_profile,
        patients::delete_account,
        patients::update_medical_info,
    ),
    components(
        schemas(
            auth::RegisterRequest,
            auth::LoginRequest,
            auth::ForgotPasswordRequest,
            auth::ResetPasswordRequest,
            auth::VerifyTokenRequest,
            auth::AuthResponse,
            auth::UserResponse,
            auth::MessageResponse,
            auth::TokenVerifyResponse,
            auth::VerifyEmailRequest,

                        // Hospitals
            hospitals::MessageResponse,

            // Patients
            patients::UpdateProfileRequest,
            patients::ProfileResponse,
            patients::DeleteAccountResponse,
            patients::UpdateMedicalInfoRequest,

            // Specialists
            specialists::MessageResponse,

        ),

    
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "User authentication endpoints"),
        (name = "patients", description = "Patient management endpoints"),
        (name = "specialists", description = "Specialist management endpoints"),
        (name = "hospitals", description = "Hospital management endpoints"),
    ),
    info(
        title = "Health Bridge API",
        version = "1.0.0",
        description = "Health Bridge REST API",
        contact(
            name = "API Support",
            email = "support@healthbridge.com"
        ),
        license(
            name = "MIT",
        )
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                ),
            )
        }
    }
}