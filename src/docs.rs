use utoipa::OpenApi;
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

                        // Hospitals
            hospitals::MessageResponse,

            // Patients
            patients::MessageResponse,

            // Specialists
            specialists::MessageResponse,

        )
    ),
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