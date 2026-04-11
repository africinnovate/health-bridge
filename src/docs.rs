use crate::handlers::{
    admin, appointments, auth, common, hospitals, notifications, patients, referrals, socials, specialists, wallets,
};
use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

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
        patients::opt_in_patient_donor,
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
        hospitals::upload_hospital_image,
        hospitals::get_donors,
        hospitals::update_donor,
        hospitals::get_donor_stats,
        hospitals::get_donor_history,
        hospitals::get_dashboard_stats,
        hospitals::get_recent_activity,
        hospitals::update_blood_inventory,
        hospitals::get_nearby_hospitals,
        specialists::create_specialist,
        specialists::get_specialist,
        specialists::get_specialists,
        specialists::update_specialist,
        specialists::add_specialty,
        specialists::list_specialties,
        specialists::upload_license,
        specialists::get_patient_profile,
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
        common::get_consultation_preference,
        common::update_consultation_preference,
        admin::admin_dashboard,
        admin::get_users,
        admin::update_specialist_status,
        admin::update_hospital_status,
        admin::update_app_config,
        // referrals::get_referral_summary,
        referrals::update_referral_link,
        wallets::get_wallet_summary,
        wallets::deposit_initialize,
        wallets::deposit_verify,
        wallets::get_banks,
        wallets::add_bank_account,
        wallets::withdraw_funds,
    ),

    components(
        schemas(
            referrals::ReferralSummaryResponse,
            referrals::RewardHistoryResponse,
            referrals::ReferralStat,
            referrals::UpdateReferralLinkPayload,
            auth::RegisterRequest,
            auth::LoginRequest,
            auth::AuthResponse,
            auth::UserResponse,
            crate::admin::dtos::ConfigActionRequest,
            crate::admin::dtos::ConfigResponse,
            crate::utils::enums::ReferralStatusEnum,
            crate::utils::enums::RewardTypeEnum,
            crate::utils::enums::Role,
            crate::utils::enums::ConsultationTypeEnum,
            wallets::WalletSummary,
            wallets::DepositInitializeResponse,
            wallets::DepositRequest,
            wallets::AddBankAccountRequest,
            wallets::WithdrawalRequest,
            crate::models::Wallet,
            crate::models::BankAccount,
            crate::models::WalletTransaction,
            crate::utils::enums::WalletTransactionStatusEnum,
            crate::utils::enums::WalletTransactionTypeEnum,
            crate::services::paystack::Bank,
            crate::admin::dtos::SpecialistActionRequest,
            crate::admin::dtos::HospitalActionRequest,
        )
    ),

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
        (name = "referrals", description = "Referral system endpoints"),
        (name = "wallets", description = "Wallet and payment endpoints"),
    ),
    info(
        title = "RubiMedik API",
        version = "1.0.0",
        description = "RubiMedik REST API",
        contact(
            name = "API Support",
            email = "[EMAIL_ADDRESS]"
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
                        .build(),
                ),
            )
        }
    }
}
