pub mod sql_types {
    use diesel::sql_types::SqlType;
     use diesel::query_builder::QueryId;

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

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "appointment_type"))]
    pub struct AppointmentTypeType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "appointment_status"))]
    pub struct AppointmentStatusType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "cancelled_by"))]
    pub struct CancelledByType;

}


diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{GenderType, RoleType};

    users (id) {
        id -> DieselUuid,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        gender -> Nullable<GenderType>,
        dob -> Nullable<Date>,
        password_hash -> Text,
        role -> RoleType,
        email_verified -> Bool,
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
        country -> Text,
        primary_phone -> Varchar,
        emergency_phone -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
        license_number -> Varchar,
        accreditation_doc_url -> Text,
        license_status -> Bool,
        has_blood_bank -> Bool,
        accepting_donors -> Bool,
        donating_operating_hours -> Nullable<Text>,
        created_at -> Timestamptz,
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
    use diesel::sql_types::Uuid as DieselUuid;
    use crate::schema::sql_types::{AppointmentTypeType, AppointmentStatusType, CancelledByType};

    appointments (id) {
        id -> Uuid,

        blood_request_id -> Uuid,
        hospital_id -> Uuid,
        user_id -> Uuid,

        appointment_type -> AppointmentTypeType,
        status -> AppointmentStatusType,

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

    specialists (user_id) {
        user_id -> DieselUuid,
        hospital_id -> Nullable<DieselUuid>,
        speciality -> Nullable<Varchar>,
        bio -> Nullable<Text>,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(patients -> users (user_id));
diesel::joinable!(email_verification_tokens -> users (user_id));
diesel::joinable!(password_reset_tokens -> users (user_id));
diesel::joinable!(specialists -> users (user_id));
diesel::joinable!(specialists -> hospitals (hospital_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    patients,
    hospitals,
    specialists,
    password_reset_tokens,
    email_verification_tokens,
);
