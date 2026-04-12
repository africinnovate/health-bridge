use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::AppError;
use crate::schema::{appointments, blood_requests, hospitals, specialists, specialties, users};
use crate::admin::dtos::{AdminPatientProfileResponse, AppointmentHistoryItem, DonationHistoryItem};
use crate::handlers::patients::fetch_patient_profile;
use crate::utils::enums::RequestStatusTypeEnum;

pub fn get_patient_detailed_profile(
    conn: &mut PgConnection,
    patient_id: Uuid,
) -> Result<AdminPatientProfileResponse, AppError> {
    // 1. Fetch base profile
    let profile = fetch_patient_profile(conn, patient_id)?;

    // 2. Fetch appointment history
    let appointments_history = appointments::table
        .inner_join(specialists::table.on(appointments::specialist_id.eq(specialists::id)))
        .inner_join(users::table.on(specialists::user_id.eq(users::id)))
        .inner_join(specialties::table.on(specialists::specialty_id.eq(specialties::id)))
        .filter(appointments::user_id.eq(patient_id))
        .order(appointments::scheduled_time.desc())
        .limit(5)
        .select((
            users::first_name,
            users::last_name,
            specialties::name,
            appointments::scheduled_time,
            appointments::status,
        ))
        .load::<(String, String, String, DateTime<Utc>, crate::utils::enums::AppointmentStatusEnum)>(conn)?
        .into_iter()
        .map(|(fname, lname, sname, time, status)| AppointmentHistoryItem {
            specialist_name: format!("Dr. {} {}", fname, lname),
            specialty: sname,
            scheduled_time: time,
            status,
        })
        .collect();

    // 3. Fetch donation history
    let donations_history = blood_requests::table
        .inner_join(hospitals::table.on(blood_requests::hospital_id.eq(hospitals::id)))
        .filter(blood_requests::donor_id.eq(patient_id))
        .order(blood_requests::created_at.desc())
        .limit(5)
        .select((
            hospitals::name,
            blood_requests::created_at,
            blood_requests::request_status,
            blood_requests::blood_type,
            blood_requests::units,
        ))
        .load::<(
            String,
            DateTime<Utc>,
            Option<RequestStatusTypeEnum>,
            Option<crate::utils::enums::BloodTypeEnum>,
            Option<i32>,
        )>(conn)?
        .into_iter()
        .map(|(hname, time, status, btype, units)| {
            DonationHistoryItem {
                hospital_name: hname,
                created_at: time,
                status: status.unwrap_or(RequestStatusTypeEnum::Confirmed),
                blood_type: btype,
                units,
            }
        })
        .collect();

    Ok(AdminPatientProfileResponse {
        profile,
        appointments: appointments_history,
        donations: donations_history,
    })
}
