use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};
use crate::handlers::{auth, hospitals, patients, appointments};

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
        hospitals::create_hospital,
        hospitals::update_hospital,
        hospitals::create_blood_request,
        hospitals::update_blood_request,
        appointments::create_appointment,
        appointments::confirm_appointment,
        appointments::reschedule_appointment,
        appointments::cancel_appointment,
        appointments::complete_appointment,
    ),
    
    components(),

    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "User authentication endpoints"),
        (name = "patients", description = "Patient management endpoints"),
        (name = "specialists", description = "Specialist management endpoints"),
        (name = "hospitals", description = "Hospital management endpoints"),
        (name = "appointments", description = "Appointment management endpoints"),
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