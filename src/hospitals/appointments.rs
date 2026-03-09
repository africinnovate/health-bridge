use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Appointment, BloodRequest, Hospital, User},
    schema::{appointments, blood_requests},
    utils::enums::{AppointmentStatusEnum, AppointmentTypeEnum, CancelledByEnum, Role},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct AppointmentResponse {
    pub appointment: Appointment,
    pub user: User,
    pub hospital: Hospital,
    pub blood_request: BloodRequest,
}

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = appointments)]
pub struct CreateAppointment {
    pub blood_request_id: Uuid,
    pub specialist_id: Uuid,
    pub appointment_type: AppointmentTypeEnum,
    pub scheduled_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AppointmentQuery {
    pub appointment_type: Option<AppointmentTypeEnum>,
    pub status: Option<AppointmentStatusEnum>,
    pub timeline: Option<String>,
}

pub fn create_appointment(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateAppointment,
) -> Result<Appointment, AppError> {
    if user.role == Role::Hospital {
        return Err(AppError::Unauthorized(
            "Hospital staff cannot create appointments".into(),
        ));
    }

    let request = blood_requests::table
        .filter(blood_requests::id.eq(payload.blood_request_id))
        .first::<BloodRequest>(conn)
        .map_err(|_| AppError::NotFound("Blood request not found".into()))?;

    diesel::insert_into(appointments::table)
        .values((
            appointments::blood_request_id.eq(payload.blood_request_id),
            appointments::hospital_id.eq(request.hospital_id),
            appointments::user_id.eq(user.id),
            appointments::specialist_id.eq(payload.specialist_id),
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
        return Err(AppError::Unauthorized(
            "Only hospitals can confirm appointments".into(),
        ));
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
            appointments::status.eq(AppointmentStatusEnum::Rescheduled),
            appointments::scheduled_time.eq(new_time),
        ))
        .returning(Appointment::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}

pub fn get_appointments(
    conn: &mut PgConnection,
    user_ctx: &User,
    filters: AppointmentQuery,
) -> Result<Vec<AppointmentResponse>, AppError> {
    use crate::schema::{
        appointments::dsl::*, blood_requests::dsl as br_dsl, hospitals::dsl as hospitals_dsl,
        users::dsl as users_dsl,
    };

    let mut query = appointments
        .inner_join(users_dsl::users.on(users_dsl::id.eq(user_id)))
        .inner_join(hospitals_dsl::hospitals.on(hospitals_dsl::id.eq(hospital_id)))
        .inner_join(br_dsl::blood_requests.on(br_dsl::id.eq(blood_request_id)))
        .into_boxed();

    // 🔐 Access control
    match user_ctx.role {
        Role::Hospital => {
            let hospital_id_owned = hospitals_dsl::hospitals
                .filter(hospitals_dsl::user_id.eq(user_ctx.id))
                .select(hospitals_dsl::id)
                .first::<Uuid>(conn)?;

            query = query.filter(hospital_id.eq(hospital_id_owned));
        }
        Role::Specialist => {
            query = query.filter(specialist_id.eq(user_ctx.id));
        }
        Role::Admin => {}
        _ => return Err(AppError::Unauthorized("Access denied".into())),
    }

    // 🔍 Filters
    if let Some(t) = filters.appointment_type {
        query = query.filter(appointment_type.eq(t));
    }

    if let Some(s) = filters.status {
        query = query.filter(status.eq(s));
    }

    if let Some(tl) = filters.timeline {
        let now = chrono::Utc::now();
        match tl.as_str() {
            "today" => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();
                query = query
                    .filter(scheduled_time.ge(start))
                    .filter(scheduled_time.le(end));
            }
            "this_week" => {
                use chrono::Datelike;
                let days_from_mon = now.weekday().num_days_from_monday();
                let start = (now - chrono::Duration::days(days_from_mon as i64))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let end = start + chrono::Duration::days(7);
                query = query
                    .filter(scheduled_time.ge(start))
                    .filter(scheduled_time.lt(end));
            }
            "this_month" => {
                use chrono::Datelike;
                let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let mut next_month = now.month() + 1;
                let mut year = now.year();
                if next_month > 12 {
                    next_month = 1;
                    year += 1;
                }
                let end = chrono::NaiveDate::from_ymd_opt(year, next_month, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                query = query
                    .filter(scheduled_time.ge(start))
                    .filter(scheduled_time.lt(end));
            }
            "upcoming" => {
                query = query.filter(scheduled_time.ge(now));
            }
            _ => {}
        }
    }

    let rows = query
        .select((
            Appointment::as_select(),
            User::as_select(),
            Hospital::as_select(),
            BloodRequest::as_select(),
        ))
        .order(created_at.desc())
        .load::<(Appointment, User, Hospital, BloodRequest)>(conn)?;

    Ok(rows
        .into_iter()
        .map(
            |(appointment, user, hospital, blood_request)| AppointmentResponse {
                appointment,
                user,
                hospital,
                blood_request,
            },
        )
        .collect())
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

    let cancelled_by = match user.role {
        Role::Hospital => CancelledByEnum::Hospital,
        Role::Donor => CancelledByEnum::Donor,
        Role::Patient => CancelledByEnum::Patient,
        _ => unreachable!(), // already guarded above
    };

    diesel::update(appointments::table.find(appointment_id))
        .set((
            appointments::status.eq(AppointmentStatusEnum::Cancelled),
            appointments::cancelled_by.eq(Some(cancelled_by)),
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
        return Err(AppError::Unauthorized(
            "Only hospitals can complete appointments".into(),
        ));
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
        // .filter(appointments::hospital_id.eq(hospital_id)) use ACL here later
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
