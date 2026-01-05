use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{User, UserSettings, NewUserSettings, Patient, MedicalInfo},
    schema::{user_settings::dsl::*, patients::dsl::*},
};


pub fn upsert_medical_info(
    conn: &mut PgConnection,
    user_id_: Uuid,
    data: MedicalInfo<'_>,
) -> Result<Patient, AppError> {
    // Check if patient record exists
    let existing = patients
        .filter(user_id.eq(user_id_))
        .first::<Patient>(conn)
        .optional()?;

    let result = match existing {
        Some(_) => {
            diesel::update(patients.filter(user_id.eq(user_id_)))
                .set((
                    blood_type.eq(data.blood_type),
                    chronic_illnesses.eq(data.chronic_illnesses),
                    allergies.eq(data.allergies),
                    hmo_number.eq(data.hmo_number),
                    emergency_contact_name.eq(data.emergency_contact_name),
                    emergency_contact_phone.eq(data.emergency_contact_phone),
                    medical_notes.eq(data.medical_notes),
                ))
                .get_result::<Patient>(conn)?
        }
        None => {
            diesel::insert_into(patients)
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
    match user_settings
        .filter(user_id.eq(user.id))
        .select(UserSettings::as_select())
        .first::<UserSettings>(conn)
    {
        Ok(settings) => Ok(settings),
        Err(diesel::result::Error::NotFound) => {
            let new_settings = NewUserSettings { user_id: user.id };

            diesel::insert_into(user_settings)
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
    diesel::update(user_settings.filter(user_id.eq(user.id)))
        .set((
            payload.appointment_reminders.map(|v| appointment_reminders.eq(v)),
            payload.specialist_recommendations.map(|v| specialist_recommendations.eq(v)),
            payload.donation_alerts.map(|v| donation_alerts.eq(v)),
            payload.account_notifications.map(|v| account_notifications.eq(v)),

            payload.email_notifications.map(|v| email_notifications.eq(v)),
            payload.sms_notifications.map(|v| sms_notifications.eq(v)),
            payload.push_notifications.map(|v| push_notifications.eq(v)),

            payload.medical_profile_visibility.map(|v| medical_profile_visibility.eq(v)),
            payload.allow_specialists_view_history.map(|v| allow_specialists_view_history.eq(v)),
            payload.allow_app_analytics.map(|v| allow_app_analytics.eq(v)),
            payload.allow_marketing_notifications.map(|v| allow_marketing_notifications.eq(v)),

            updated_at.eq(diesel::dsl::now),
        ))
        .get_result::<UserSettings>(conn)
        .map_err(AppError::from)
}
