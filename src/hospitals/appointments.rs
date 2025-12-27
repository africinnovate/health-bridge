use diesel::prelude::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Appointment, BloodRequest, User},
    schema::{appointments, blood_requests},
    utils::enums::{Role, AppointmentStatusEnum, AppointmentTypeEnum},

};



#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = appointments)]
pub struct CreateAppointment {
    pub blood_request_id: Uuid,
    pub user_id: Uuid, // donor or patient
    pub appointment_type: AppointmentTypeEnum,
    pub scheduled_time: DateTime<Utc>,
}


pub fn create_appointment(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateAppointment,
) -> Result<Appointment, AppError> {
    if user.role == Role::Hospital {
        return Err(AppError::Unauthorized("Hospital staff cannot create appointments".into()));
    }

    let request = blood_requests::table
        .filter(blood_requests::id.eq(payload.blood_request_id))
        .filter(blood_requests::hospital_id.eq(user.id))
        .first::<BloodRequest>(conn)?;

    diesel::insert_into(appointments::table)
        .values((
            appointments::blood_request_id.eq(payload.blood_request_id),
            appointments::hospital_id.eq(request.hospital_id),
            appointments::user_id.eq(payload.user_id),
            appointments::appointment_type.eq(payload.appointment_type),
            appointments::status.eq(AppointmentStatusEnum::Created),
            appointments::scheduled_time.eq(payload.scheduled_time),
        ))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}


pub fn confirm_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    user: &User,
) -> Result<Appointment, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized("Only hospitals can confirm appointments".into()));
    }

    assert_hospital_owns_appointment(conn, appointment_id, user.id)?;

    diesel::update(appointments::table.find(appointment_id))
        .set(appointments::status.eq(AppointmentStatusEnum::Confirmed))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}


pub fn reschedule_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    user: &User,
    new_time: DateTime<Utc>,
) -> Result<Appointment, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized("Only hospitals can reschedule appointments".into()));
    }

    assert_hospital_owns_appointment(conn, appointment_id, user.id)?;

    diesel::update(appointments::table.find(appointment_id))
        .set((
            appointments::status.eq(AppointmentStatusEnum::Rescheduled),
            appointments::scheduled_time.eq(new_time),
        ))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}


pub fn cancel_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    user: &User,
    reason: Option<String>,
) -> Result<Appointment, AppError> {
    match user.role {
        Role::Hospital => {
            assert_hospital_owns_appointment(conn, appointment_id, user.id)?;
        }
        Role::Donor | Role::Patient => {
            assert_user_owns_appointment(conn, appointment_id, user.id)?;
        }
        _ => return Err(AppError::Unauthorized("Invalid role".into())),
    }

    diesel::update(appointments::table.find(appointment_id))
        .set((
            appointments::status.eq(AppointmentStatusEnum::Cancelled),
            appointments::cancelled_by_id.eq(Some(user.id)),
            appointments::cancelled_reason.eq(reason),
            appointments::cancelled_at.eq(Some(Utc::now())),
        ))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}

pub fn complete_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    user: &User,
) -> Result<Appointment, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized("Only hospitals can complete appointments".into()));
    }

    assert_hospital_owns_appointment(conn, appointment_id, user.id)?;

    diesel::update(appointments::table.find(appointment_id))
        .set(appointments::status.eq(AppointmentStatusEnum::Completed))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}



fn assert_hospital_owns_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    hospital_id: Uuid,
) -> Result<Appointment, AppError> {
    appointments::table
        .filter(appointments::id.eq(appointment_id))
        .filter(appointments::hospital_id.eq(hospital_id))
        .first::<Appointment>(conn)
        .map_err(|_| AppError::Unauthorized("Appointment not owned by hospital".into()))
}

fn assert_user_owns_appointment(
    conn: &mut PgConnection,
    appointment_id: Uuid,
    user_id: Uuid,
) -> Result<Appointment, AppError> {
    appointments::table
        .filter(appointments::id.eq(appointment_id))
        .filter(appointments::user_id.eq(user_id))
        .first::<Appointment>(conn)
        .map_err(|_| AppError::Unauthorized("Appointment not owned by user".into()))
}
