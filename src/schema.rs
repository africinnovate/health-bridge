diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    users (id) {
        id -> DieselUuid,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        gender -> Nullable<Text>,
        dob -> Nullable<Date>,
        password_hash -> Text,
        role -> Text,
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
        medical_notes -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel::sql_types::Uuid as DieselUuid;

    hospitals (id) {
        id -> DieselUuid,
        name -> Varchar,
        address -> Nullable<Text>,
        phone -> Nullable<Varchar>,
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
