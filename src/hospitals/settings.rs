use diesel::prelude::*;
use uuid::Uuid;
use crate::{
    error::AppError, 
    handlers::hospitals::UpdateHospitalSettingsRequest, 
    models::{HospitalSettings, NewHospitalSettings}, 
    schema::hospital_settings::dsl as hs
};

pub fn get_or_create_hospital_settings(
    conn: &mut PgConnection,
    hospital_id_: Uuid,
) -> Result<HospitalSettings, AppError> {
    match hs::hospital_settings
        .filter(hs::hospital_id.eq(hospital_id_))
        .select(HospitalSettings::as_select())
        .first::<HospitalSettings>(conn)
    {
        Ok(settings) => Ok(settings),
        Err(diesel::result::Error::NotFound) => {
            let new_settings = NewHospitalSettings {
                hospital_id: hospital_id_,
            };

            diesel::insert_into(hs::hospital_settings)
                .values(&new_settings)
                .get_result(conn)
                .map_err(AppError::from)
        }
        Err(e) => Err(AppError::from(e)),
    }
}


pub fn update_hospital_settings(
    conn: &mut PgConnection,
    hospital_id_: Uuid,
    payload: UpdateHospitalSettingsRequest,
) -> Result<HospitalSettings, AppError> {
    diesel::update(
        hs::hospital_settings
            .filter(hs::hospital_id.eq(hospital_id_)),
    )
    .set((
        payload.donation_requests.map(|v| hs::donation_requests.eq(v)),
        payload
            .new_donor_appointments
            .map(|v| hs::new_donor_appointments.eq(v)),
        payload
            .donor_appointment_reminders
            .map(|v| hs::donor_appointment_reminders.eq(v)),
        payload.login_alerts.map(|v| hs::login_alerts.eq(v)),
        payload
            .account_notifications
            .map(|v| hs::account_notifications.eq(v)),
        payload.email_notifications.map(|v| hs::email_notifications.eq(v)),
        payload.sms_notifications.map(|v| hs::sms_notifications.eq(v)),
        payload.push_notifications.map(|v| hs::push_notifications.eq(v)),
        hs::updated_at.eq(diesel::dsl::now),
    ))
    .get_result(conn)
    .map_err(AppError::from)
}
