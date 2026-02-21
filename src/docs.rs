use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};
use crate::handlers::{auth, hospitals, patients, appointments, specialists, admin, notifications, socials, common};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register,
        auth::login,
        auth::forgot_password,
        auth::reset_password,
        auth::update_password,
        auth::verify_token,
        auth::verify_email,
        auth::resend_verification_code,
        auth::delete_account,
        auth::refresh_token,
        auth::logout,
        auth::logout_all_devices,
        patients::get_profile,
        patients::update_profile,
        patients::delete_account,
        patients::update_medical_info,
        hospitals::get_hospitals,
        hospitals::get_user_hospitals,
        hospitals::get_hospital_by_id,
        hospitals::create_hospital,
        hospitals::update_hospital,
        hospitals::create_blood_request,
        hospitals::get_blood_requests,
        hospitals::update_blood_request,
        hospitals::get_hospital_settings,
        hospitals::update_hospital_settings,
        hospitals::delete_hospital,
        hospitals::upload_accreditation_doc,
        specialists::create_specialist,
        specialists::get_specialist,
        specialists::get_specialists,
        specialists::update_specialist,
        specialists::add_specialty,
        specialists::list_specialties,
        specialists::upload_license,
        appointments::get_appointments,
        appointments::create_appointment,
        appointments::confirm_appointment,
        appointments::reschedule_appointment,
        appointments::cancel_appointment,
        appointments::complete_appointment,
        notifications::get_notifications,
        notifications::mark_notification_as_read,
        notifications::mark_all_notifications_as_read,
        socials::social_login,
        common::get_user_settings,
        common::update_user_settings,
        common::upload_image,
        admin::admin_dashboard,
        admin::get_users,
        admin::update_specialist_status,
        admin::update_hospital_status,
    ),
    
    components(),

    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "User authentication endpoints"),
        (name = "patients", description = "Patient management endpoints"),
        (name = "specialists", description = "Specialist management endpoints"),
        (name = "hospitals", description = "Hospital management endpoints"),
        (name = "appointments", description = "Appointment management endpoints"),
        (name = "notifications", description = "Notification management endpoints"),
        (name = "settings", description = "User settings endpoints"),
        (name = "admin", description = "Admin management endpoints"),
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