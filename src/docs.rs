use utoipa::OpenApi;
use crate::handlers::users;

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
        title = "EMR API",
        version = "1.0.0",
        description = "Electronic Medical Records API",
        contact(
            name = "API Support",
            email = "support@emr-api.com"
        ),
        license(
            name = "MIT",
        )
    )
)]
pub struct ApiDoc;