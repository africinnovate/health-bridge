use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid;
use utoipa::ToSchema;
use serde::Serialize;

use crate::models::SpecialistAvailability;
use crate::services::mail::MailService;
use crate::utils::enums::{ConsultationTypeEnum, DaysOfWeekEnum, Gender};
use crate::{
    error::AppError,
    models::{Specialist, User},
    schema::specialists,
    schema::specialist_availabilities,
    utils::enums::Role,
    handlers::specialists::{CreateSpecialistRequest, UpdateSpecialistWithAvailability},
};

#[derive(Debug, Serialize, ToSchema)]
pub struct SpecialistResponse {
    pub id: Uuid,

    // user fields
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,

    // specialist fields
    pub hospital_id: Option<Uuid>,
    pub specialty_id: Uuid,
    pub bio: Option<String>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: ConsultationTypeEnum,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<String>,
    pub secondary_phone: Option<String>,
    pub languages_spoken: Option<String>,
    pub verified: bool,
    pub suspended: bool,
    pub created_at: DateTime<Utc>,

    // availability
    pub availability: Vec<SpecialistAvailabilityResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpecialistAvailabilityResponse {
    pub day_of_week: DaysOfWeekEnum,
    pub opens_at: chrono::NaiveTime,
    pub closes_at: chrono::NaiveTime,
}


pub fn get_specialist_with_user(
    conn: &mut PgConnection,
    specialist_id: Uuid,
) -> Result<SpecialistResponse, AppError> {

    use crate::schema::{specialists, users, specialist_availabilities};

    let (specialist, user) = specialists::table
        .inner_join(users::table.on(users::id.eq(specialists::user_id)))
        .filter(specialists::id.eq(specialist_id))
        .select((Specialist::as_select(), User::as_select()))
        .first::<(Specialist, User)>(conn)
        .map_err(|_| AppError::NotFound("Specialist not found".into()))?;

    let availability = specialist_availabilities::table
    .filter(specialist_availabilities::specialist_id.eq(specialist.id))
    .select(SpecialistAvailability::as_select())
    .load::<SpecialistAvailability>(conn)?
    .into_iter()
    .map(|a| SpecialistAvailabilityResponse {
        day_of_week: a.day_of_week,
        opens_at: a.opens_at,
        closes_at: a.closes_at,
    })
    .collect();

    Ok(SpecialistResponse {
        id: specialist.id,

        user_id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        phone: user.phone,
        gender: user.gender,

        hospital_id: specialist.hospital_id,
        specialty_id: specialist.specialty_id,
        bio: specialist.bio,
        years_of_experience: specialist.years_of_experience,
        consultation_type: specialist.consultation_type,
        session_duration_minutes: specialist.session_duration_minutes,
        primary_phone: specialist.primary_phone,
        secondary_phone: specialist.secondary_phone,
        languages_spoken: specialist.languages_spoken,
        verified: specialist.verified,
        suspended: specialist.suspended,
        created_at: specialist.created_at,

        availability,
    })
}



pub fn create_specialist(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateSpecialistRequest,
) -> Result<Specialist, AppError> {

    if user.role != Role::Specialist {
        return Err(AppError::Unauthorized("Only specialists allowed".into()));
    }

    let specialist = diesel::insert_into(specialists::table)
        .values((
            specialists::user_id.eq(user.id),
            specialists::specialty_id.eq(payload.specialty_id),
            specialists::consultation_type.eq(payload.consultation_type),
            specialists::bio.eq(payload.bio),
            specialists::years_of_experience.eq(payload.years_of_experience),
            specialists::session_duration_minutes.eq(payload.session_duration_minutes),
            specialists::primary_phone.eq(payload.primary_phone),
            specialists::secondary_phone.eq(payload.secondary_phone),
            specialists::languages_spoken.eq(payload.languages_spoken),
        ))
        .returning(Specialist::as_select())
        .get_result(conn)?;

    for slot in payload.availabilities {
        diesel::insert_into(specialist_availabilities::table)
            .values((
                specialist_availabilities::specialist_id.eq(specialist.id),
                specialist_availabilities::day_of_week.eq(slot.day_of_week),
                specialist_availabilities::opens_at.eq(slot.opens_at),
                specialist_availabilities::closes_at.eq(slot.closes_at),
            ))
            .execute(conn)?;
    }

    Ok(specialist)
}


pub fn update_specialist(
    conn: &mut PgConnection,
    specialist_id: Uuid,
    user: &User,
    payload: UpdateSpecialistWithAvailability,
) -> Result<Specialist, AppError> {

    // Update main specialist fields
    let updated = diesel::update(
        specialists::table
            .filter(specialists::id.eq(specialist_id))
            .filter(specialists::user_id.eq(user.id)),
    )
    .set(&payload.specialist)  // only the table columns
    .returning(Specialist::as_select())
    .get_result::<Specialist>(conn)
    .optional()?
    .ok_or_else(|| AppError::NotFound("Specialist not found".into()))?;

    // Handle availabilities separately
    if let Some(availabilities) = payload.availabilities {
        diesel::delete(
            specialist_availabilities::table
                .filter(specialist_availabilities::specialist_id.eq(specialist_id)),
        )
        .execute(conn)?;

        for slot in availabilities {
            diesel::insert_into(specialist_availabilities::table)
                .values((
                    specialist_availabilities::specialist_id.eq(specialist_id),
                    specialist_availabilities::day_of_week.eq(slot.day_of_week),
                    specialist_availabilities::opens_at.eq(slot.opens_at),
                    specialist_availabilities::closes_at.eq(slot.closes_at),
                ))
                .execute(conn)?;
        }
    }

    Ok(updated)
}

// pub fn update_specialist(
//     conn: &mut PgConnection,
//     specialist_id: Uuid,
//     user: &User,
//     payload: UpdateSpecialistRequest,
// ) -> Result<Specialist, AppError> {

//     let updated = diesel::update(
//         specialists::table
//             .filter(specialists::id.eq(specialist_id))
//             .filter(specialists::user_id.eq(user.id)),
//     )
//     .set(payload)
//     .returning(Specialist::as_select())
//     .get_result::<Specialist>(conn)
//     .optional()?;

//     updated.ok_or_else(|| AppError::NotFound("Specialist not found".into()))
// }
