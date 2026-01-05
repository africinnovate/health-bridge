use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    error::AppError, 
    handlers::common::UpdateUserSettingsRequest, 
    models::{MedicalInfo, NewUserSettings, Patient, User, UserSettings}, 
    schema::{
        patients::dsl as patients_dsl,
        user_settings::dsl as settings_dsl,
    },
};

pub fn upsert_medical_info(
    conn: &mut PgConnection,
    user_id_: Uuid,
    data: MedicalInfo<'_>,
) -> Result<Patient, AppError> {
    let existing = patients_dsl::patients
        .filter(patients_dsl::user_id.eq(user_id_))
        .first::<Patient>(conn)
        .optional()?;

    let result = match existing {
        Some(_) => {
            diesel::update(
                patients_dsl::patients
                    .filter(patients_dsl::user_id.eq(user_id_)),
            )
            .set((
                patients_dsl::blood_type.eq(data.blood_type),
                patients_dsl::chronic_illnesses.eq(data.chronic_illnesses),
                patients_dsl::allergies.eq(data.allergies),
                patients_dsl::hmo_number.eq(data.hmo_number),
                patients_dsl::emergency_contact_name.eq(data.emergency_contact_name),
                patients_dsl::emergency_contact_phone.eq(data.emergency_contact_phone),
                patients_dsl::medical_notes.eq(data.medical_notes),
            ))
            .get_result::<Patient>(conn)?
        }
        None => {
            diesel::insert_into(patients_dsl::patients)
                .values(&data)
                .get_result::<Patient>(conn)?
        }
    };

    Ok(result)
}

pub fn get_or_create_user_settings(
    conn: &mut PgConnection,
    user: &User,
) -> Result<UserSettings, AppError> {
    match settings_dsl::user_settings
        .filter(settings_dsl::user_id.eq(user.id))
        .first::<UserSettings>(conn)
    {
        Ok(settings) => Ok(settings),
        Err(diesel::result::Error::NotFound) => {
            let new_settings = NewUserSettings { user_id: user.id };

            diesel::insert_into(settings_dsl::user_settings)
                .values(&new_settings)
                .get_result::<UserSettings>(conn)
                .map_err(AppError::from)
        }
        Err(e) => Err(AppError::from(e)),
    }
}

pub fn update_user_settings(
    conn: &mut PgConnection,
    user: &User,
    payload: UpdateUserSettingsRequest,
) -> Result<UserSettings, AppError> {
    diesel::update(
        settings_dsl::user_settings
            .filter(settings_dsl::user_id.eq(user.id)),
    )
    .set((
        payload.appointment_reminders
            .map(|v| settings_dsl::appointment_reminders.eq(v)),
        payload.specialist_recommendations
            .map(|v| settings_dsl::specialist_recommendations.eq(v)),
        payload.donation_alerts
            .map(|v| settings_dsl::donation_alerts.eq(v)),
        payload.account_notifications
            .map(|v| settings_dsl::account_notifications.eq(v)),

        payload.email_notifications
            .map(|v| settings_dsl::email_notifications.eq(v)),
        payload.sms_notifications
            .map(|v| settings_dsl::sms_notifications.eq(v)),
        payload.push_notifications
            .map(|v| settings_dsl::push_notifications.eq(v)),

        payload.medical_profile_visibility
            .map(|v| settings_dsl::medical_profile_visibility.eq(v)),
        payload.allow_specialists_view_history
            .map(|v| settings_dsl::allow_specialists_view_history.eq(v)),
        payload.allow_app_analytics
            .map(|v| settings_dsl::allow_app_analytics.eq(v)),
        payload.allow_marketing_notifications
            .map(|v| settings_dsl::allow_marketing_notifications.eq(v)),

        settings_dsl::updated_at.eq(diesel::dsl::now),
    ))
    .get_result::<UserSettings>(conn)
    .map_err(AppError::from)
}