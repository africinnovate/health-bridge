pub mod sql_types {
    use diesel::query_builder::QueryId;
    use diesel::sql_types::SqlType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "gender_type"))]
    pub struct GenderType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "role_type"))]
    pub struct RoleType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "hospital_type"))]
    pub struct HospitalType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "blood_group"))]
    pub struct BloodGroupType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "genotype"))]
    pub struct GenotypeType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "blood_type"))]
    pub struct BloodType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "urgency_type"))]
    pub struct UrgencyType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "blood_request_status_type"))]
    pub struct BloodRequestStatusType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "timeline_type"))]
    pub struct TimelineType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "appointment_type"))]
    pub struct AppointmentTypeType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "appointment_status"))]
    pub struct AppointmentStatusType;

    #[derive(SqlType)]
    #[diesel(postgres_type(name = "cancelled_by"))]
    pub struct CancelledByType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "consultation_type"))]
    pub struct ConsultationType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "days_of_week_type"))]
    pub struct DaysOfWeekType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "action_type"))]
    pub struct ActionType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "notification_category"))]
    pub struct NotificationCategoryType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "referral_status"))]
    pub struct ReferralStatusType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "reward_type"))]
    pub struct RewardTypeType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "wallet_transaction_type"))]
    pub struct WalletTransactionType;

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "wallet_transaction_status"))]
    pub struct WalletTransactionStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{GenderType, RoleType, ConsultationType};

    users (id) {
        id -> DieselUuid,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        gender -> Nullable<GenderType>,
        image_url -> Nullable<Varchar>,
        address -> Nullable<Text>,
        city -> Nullable<Varchar>,
        state -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        dob -> Nullable<Date>,
        password_hash -> Text,
        role -> RoleType,
        email_verified -> Bool,
        note -> Nullable<Text>,
        eligible_to_donate -> Bool,
        consultation_preference -> Nullable<ConsultationType>,
        referral_code -> Varchar,
        referral_link -> Nullable<Text>,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    app_configs (id) {
        id -> DieselUuid,
        key -> Varchar,
        value -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::ReferralStatusType;

    referrals (id) {
        id -> DieselUuid,
        referrer_id -> DieselUuid,
        referred_user_id -> DieselUuid,
        referral_code -> Varchar,
        status -> ReferralStatusType,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::RewardTypeType;

    referral_rewards (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        referral_id -> Nullable<DieselUuid>,
        points -> Int4,
        reward_type -> RewardTypeType,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    email_verification_tokens (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        code -> Varchar,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    password_reset_tokens (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        token -> Text,
        expires_at -> Timestamptz,
        used -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    patients (user_id) {
        user_id -> DieselUuid,
        blood_type -> Nullable<Varchar>,
        chronic_illnesses -> Nullable<Text>,
        allergies -> Nullable<Text>,
        medications -> Nullable<Text>,
        existing_conditions -> Nullable<Text>,
        primary_physician -> Nullable<DieselUuid>,
        hmo_number -> Nullable<Varchar>,
        emergency_contact_name -> Nullable<Varchar>,
        emergency_contact_phone -> Nullable<Varchar>,
        medical_notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{HospitalType};

    hospitals (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        name -> Varchar,
        hospital_type -> Nullable<HospitalType>,
        address -> Text,
        city -> Text,
        state -> Text,
        country -> Text,
        primary_phone -> Varchar,
        emergency_phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        email_verified -> Bool,
        license_number -> Varchar,
        accreditation_doc_url -> Text,
        profile_image -> Nullable<Varchar>,
        license_status -> Bool,
        has_blood_bank -> Bool,
        accepting_donors -> Bool,
        donating_operating_hours -> Nullable<Text>,
        created_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::schema::sql_types::*;
    use crate::schema::sql_types::{BloodType, UrgencyType, BloodRequestStatusType, TimelineType};

    blood_requests (id) {
        id -> Uuid,
        hospital_id -> Uuid,
        donor_id -> Nullable<Uuid>,
        recipient_id -> Nullable<Uuid>,
        ref_id -> Text,
        units -> Nullable<Int4>,
        blood_type -> Nullable<BloodType>,
        urgency -> Nullable<UrgencyType>,
        timeline_status -> Nullable<TimelineType>,
        request_status -> Nullable<BloodRequestStatusType>,
        request_reason -> Nullable<Text>,
        note -> Nullable<Text>,
        cancelled_by -> Nullable<Uuid>,
        cancelled_at -> Nullable<Timestamptz>,
        cancelled_reason -> Nullable<Text>,
        preferred_time -> Nullable<Timestamptz>,
        donated_at -> Nullable<Timestamptz>,
        administered_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BloodType;

    hospital_blood_inventories (id) {
        id -> Uuid,
        hospital_id -> Uuid,
        blood_type -> BloodType,
        units_available -> Int4,
        bank_capacity -> Int4,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{AppointmentTypeType, AppointmentStatusType, CancelledByType};

    appointments (id) {
        id -> Uuid,

        blood_request_id -> Nullable<Uuid>,
        hospital_id -> Nullable<Uuid>,
        user_id -> Uuid,
        specialist_id -> Uuid,

        appointment_type -> AppointmentTypeType,
        status -> AppointmentStatusType,
        notes -> Nullable<Text>,

        scheduled_time -> Timestamptz,
        previous_time -> Nullable<Timestamptz>,

        cancelled_by -> Nullable<CancelledByType>,
        cancelled_by_id -> Nullable<Uuid>,
        cancelled_reason -> Nullable<Text>,
        cancelled_at -> Nullable<Timestamptz>,


        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    specialties (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{ConsultationType};

    specialists (id) {
        id -> Uuid,
        user_id -> Uuid,
        hospital_id -> Nullable<Uuid>,
        specialty_id -> Uuid,
        bio -> Nullable<Text>,
        years_of_experience -> Nullable<Int4>,
        consultation_type -> ConsultationType,
        session_duration_minutes -> Nullable<Int4>,
        primary_phone -> Nullable<Varchar>,
        secondary_phone -> Nullable<Varchar>,
        languages_spoken -> Nullable<Varchar>,
        country -> Nullable<Varchar>,
        time_zone -> Nullable<Varchar>,
        license_url -> Nullable<Text>,
        verified -> Bool,
        suspended -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::DaysOfWeekType;

    specialist_availabilities (id) {
        id -> Uuid,
        specialist_id -> Uuid,
        day_of_week -> DaysOfWeekType,
        opens_at -> Time,
        closes_at -> Time,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::ActionType;

    admin_audit_logs (id) {
        id -> Uuid,
        admin_id -> Uuid,
        target_type -> Varchar,
        target_id -> Uuid,
        action_type -> ActionType,
        reason -> Nullable<Text>,
        metadata -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::NotificationCategoryType;

    notifications (id) {
        id -> Uuid,
        user_id -> Uuid,
        category -> NotificationCategoryType,
        title -> Varchar,
        message -> Text,
        related_id -> Nullable<Uuid>,
        related_type -> Nullable<Varchar>,
        metadata -> Nullable<Text>,
        is_read -> Bool,
        created_by -> Nullable<Uuid>,
        created_at -> Timestamptz,
        read_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    social_accounts (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        provider -> Varchar,
        provider_user_id -> Varchar,
        email -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_settings (id) {
        id -> Uuid,
        user_id -> Uuid,

        appointment_reminders -> Bool,
        specialist_recommendations -> Bool,
        donation_alerts -> Bool,
        account_notifications -> Bool,

        email_notifications -> Bool,
        sms_notifications -> Bool,
        push_notifications -> Bool,

        medical_profile_visibility -> Text,
        allow_specialists_view_history -> Bool,
        allow_app_analytics -> Bool,
        allow_marketing_notifications -> Bool,

        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    hospital_settings (id) {
        id -> Uuid,
        hospital_id -> Uuid,

        donation_requests -> Bool,

        new_donor_appointments -> Bool,
        donor_appointment_reminders -> Bool,

        login_alerts -> Bool,
        account_notifications -> Bool,

        email_notifications -> Bool,
        sms_notifications -> Bool,
        push_notifications -> Bool,

        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    refresh_tokens (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        token -> Text,
        expires_at -> Timestamptz,
        revoked -> Bool,
        created_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    wallets (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        balance -> Numeric,
        currency -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    bank_accounts (id) {
        id -> DieselUuid,
        user_id -> DieselUuid,
        account_number -> Varchar,
        bank_code -> Varchar,
        bank_name -> Varchar,
        account_name -> Varchar,
        recipient_code -> Varchar,
        is_default -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{WalletTransactionType, WalletTransactionStatus};

    wallet_transactions (id) {
        id -> DieselUuid,
        wallet_id -> DieselUuid,
        amount -> Numeric,
        transaction_type -> WalletTransactionType,
        status -> WalletTransactionStatus,
        reference -> Varchar,
        provider -> Varchar,
        description -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    consultation_types (id) {
        id -> DieselUuid,
        name -> Varchar,
        description -> Nullable<Text>,
        duration_minutes -> Int4,
        base_price -> Numeric,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    consultation_benefits (id) {
        id -> DieselUuid,
        title -> Varchar,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    consultation_type_benefits (consultation_type_id, consultation_benefit_id) {
        consultation_type_id -> DieselUuid,
        consultation_benefit_id -> DieselUuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    consultation_packages (id) {
        id -> DieselUuid,
        specialist_id -> DieselUuid,
        consultation_type_id -> DieselUuid,
        name -> Varchar,
        description -> Nullable<Text>,
        custom_price -> Nullable<Numeric>,
        custom_duration_minutes -> Nullable<Int4>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    consultation_package_benefits (id) {
        id -> DieselUuid,
        package_id -> DieselUuid,
        consultation_benefit_id -> Nullable<DieselUuid>,
        custom_title -> Nullable<Varchar>,
        custom_description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(patients -> users (user_id));
diesel::joinable!(email_verification_tokens -> users (user_id));
diesel::joinable!(password_reset_tokens -> users (user_id));
diesel::joinable!(specialists -> users (user_id));
diesel::joinable!(specialists -> specialties (specialty_id));
diesel::joinable!(specialist_availabilities -> specialists (specialist_id));
diesel::joinable!(specialists -> hospitals (hospital_id));
diesel::joinable!(hospital_blood_inventories -> hospitals (hospital_id));
diesel::joinable!(appointments -> users (specialist_id));
diesel::joinable!(appointments -> hospitals (hospital_id));
diesel::joinable!(appointments -> blood_requests (blood_request_id));
diesel::joinable!(admin_audit_logs -> users (admin_id));
diesel::joinable!(notifications -> users (user_id));
diesel::joinable!(social_accounts -> users (user_id));
diesel::joinable!(user_settings -> users (user_id));
diesel::joinable!(hospital_settings -> hospitals (hospital_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(referrals -> users (referrer_id));
diesel::joinable!(referral_rewards -> users (user_id));
diesel::joinable!(referral_rewards -> referrals (referral_id));
diesel::joinable!(wallets -> users (user_id));
diesel::joinable!(bank_accounts -> users (user_id));
diesel::joinable!(wallet_transactions -> wallets (wallet_id));
diesel::joinable!(consultation_type_benefits -> consultation_types (consultation_type_id));
diesel::joinable!(consultation_type_benefits -> consultation_benefits (consultation_benefit_id));
diesel::joinable!(consultation_packages -> specialists (specialist_id));
diesel::joinable!(consultation_packages -> consultation_types (consultation_type_id));
diesel::joinable!(consultation_package_benefits -> consultation_packages (package_id));
diesel::joinable!(consultation_package_benefits -> consultation_benefits (consultation_benefit_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    patients,
    hospitals,
    specialties,
    specialists,
    password_reset_tokens,
    specialist_availabilities,
    email_verification_tokens,
    appointments,
    blood_requests,
    admin_audit_logs,
    notifications,
    social_accounts,
    user_settings,
    hospital_settings,
    hospital_blood_inventories,
    refresh_tokens,
    app_configs,
    referrals,
    referral_rewards,
    wallets,
    bank_accounts,
    wallet_transactions,
    consultation_types,
    consultation_benefits,
    consultation_type_benefits,
    consultation_packages,
    consultation_package_benefits,
);
