use utoipa::OpenApi;
use crate::handlers::users;
use crate::handlers::hospitals;
use crate::handlers::patients;
use crate::handlers::specialists;

#[derive(OpenApi)]
#[openapi(
    paths(
        users::get_users,
        users::register,
        users::login,
        users::forgot_password,
        users::reset_password,
        users::verify_token,
    ),
    components(
        schemas(
            users::RegisterRequest,
            users::LoginRequest,
            users::ForgotPasswordRequest,
            users::ResetPasswordRequest,
            users::VerifyTokenRequest,
            users::AuthResponse,
            users::UserResponse,
            users::MessageResponse,
            users::TokenVerifyResponse,

                        // Hospitals
            hospitals::MessageResponse,

            // Patients
            patients::MessageResponse,

            // Specialists
            specialists::MessageResponse,

        )
    ),
    tags(
        (name = "authentication", description = "User authentication endpoints"),
        (name = "users", description = "User management endpoints"),
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