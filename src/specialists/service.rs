use chrono::{DateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::SpecialistAvailability;
use crate::schema::{specialties, users};
use crate::utils::enums::{ConsultationTypeEnum, DaysOfWeekEnum, Gender};
use crate::{
    error::AppError,
    handlers::specialists::{CreateSpecialistRequest, UpdateSpecialistWithAvailability},
    models::{Specialist, User},
    schema::specialist_availabilities,
    schema::specialists,
    utils::enums::Role,
};

#[derive(AsChangeset)]
#[diesel(table_name = specialists)]
pub struct SpecialistChangeset<'a> {
    pub bio: Option<&'a str>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: Option<ConsultationTypeEnum>,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<&'a str>,
    pub secondary_phone: Option<&'a str>,
    pub languages_spoken: Option<&'a str>,
    pub country: Option<&'a str>,
    pub time_zone: Option<&'a str>,
    pub license_url: Option<&'a str>,
    pub suspended: Option<bool>,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub struct UserChangeset<'a> {
    pub first_name: Option<&'a str>,
    pub last_name: Option<&'a str>,
    pub address: Option<&'a str>,
    pub city: Option<&'a str>,
    pub state: Option<&'a str>,
    pub country: Option<&'a str>,
}

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
    pub city: Option<String>,
    pub state: Option<String>,
    pub image_url: Option<String>,
    pub email_verified: bool,
    pub consultation_preference: Option<ConsultationTypeEnum>,

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
    pub license_url: Option<String>,
    pub country: Option<String>,
    pub time_zone: Option<String>,
    pub verified: bool,
    pub suspended: bool,
    pub created_at: DateTime<Utc>,

    // availability
    pub availability: Vec<SpecialistAvailabilityResponse>,
}

#[derive(Debug, Deserialize)]
pub struct SpecialistFilters {
    pub verified: Option<bool>,
    pub suspended: Option<bool>,
    pub specialty_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpecialistAvailabilityResponse {
    pub day_of_week: DaysOfWeekEnum,
    pub opens_at: chrono::NaiveTime,
    pub closes_at: chrono::NaiveTime,
}

pub fn get_specialist_with_user(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<SpecialistResponse, AppError> {
    use crate::schema::{specialist_availabilities, specialists, users};
    let (specialist, user) = specialists::table
        .inner_join(users::table.on(users::id.eq(specialists::user_id)))
        .filter(users::id.eq(user_id))
        .filter(users::deleted_at.is_null())
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
        city: user.city,
        state: user.state,
        image_url: user.image_url,
        email_verified: user.email_verified,
        consultation_preference: user.consultation_preference,

        hospital_id: specialist.hospital_id,
        specialty_id: specialist.specialty_id,
        bio: specialist.bio,
        years_of_experience: specialist.years_of_experience,
        consultation_type: specialist.consultation_type,
        session_duration_minutes: specialist.session_duration_minutes,
        primary_phone: specialist.primary_phone,
        secondary_phone: specialist.secondary_phone,
        languages_spoken: specialist.languages_spoken,
        license_url: specialist.license_url,
        country: specialist.country,
        time_zone: specialist.time_zone,
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

    // check if email is verified
    if !user.email_verified {
        return Err(AppError::Unauthorized("Email not verified".into()));
    }

    let specialty_exists = specialties::table
        .find(payload.specialty_id)
        .select(specialties::id)
        .first::<Uuid>(conn)
        .optional()?;

    if specialty_exists.is_none() {
        return Err(AppError::BadRequest(
            "Invalid specialty_id: specialty does not exist".into(),
        ));
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
            specialists::license_url.eq(payload.license_url),
            specialists::country.eq(payload.country),
            specialists::time_zone.eq(payload.time_zone),
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

pub fn get_specialists(
    conn: &mut PgConnection,
    filters: SpecialistFilters,
) -> Result<Vec<(Specialist, User, Vec<SpecialistAvailability>)>, AppError> {
    let mut query = specialists::table
        .inner_join(users::table.on(users::id.eq(specialists::user_id)))
        .into_boxed();

    if let Some(verified) = filters.verified {
        query = query.filter(specialists::verified.eq(verified));
    }

    if let Some(suspended) = filters.suspended {
        query = query.filter(specialists::suspended.eq(suspended));
    }

    if let Some(specialty_id) = filters.specialty_id {
        query = query.filter(specialists::specialty_id.eq(specialty_id));
    }

    let rows = query
        .filter(users::deleted_at.is_null())
        .select((Specialist::as_select(), User::as_select()))
        .load::<(Specialist, User)>(conn)?;

    let specialist_ids: Vec<Uuid> = rows.iter().map(|(s, _)| s.id).collect();

    let availability_map = specialist_availabilities::table
        .filter(specialist_availabilities::specialist_id.eq_any(&specialist_ids))
        .load::<SpecialistAvailability>(conn)?
        .into_iter()
        .fold(
            std::collections::HashMap::<Uuid, Vec<SpecialistAvailability>>::new(),
            |mut acc, a| {
                acc.entry(a.specialist_id).or_default().push(a);
                acc
            },
        );

    Ok(rows
        .into_iter()
        .map(|(specialist, user)| {
            let availability = availability_map
                .get(&specialist.id)
                .cloned()
                .unwrap_or_default();

            (specialist, user, availability)
        })
        .collect())
}

pub fn update_specialist(
    conn: &mut PgConnection,
    specialist_id: Uuid,
    user: &User,
    payload: UpdateSpecialistWithAvailability,
) -> Result<Specialist, AppError> {
    // Convert payload to changeset
    let specialist_changeset = SpecialistChangeset {
        bio: payload.specialist.bio.as_deref(),
        years_of_experience: payload.specialist.years_of_experience,
        consultation_type: payload.specialist.consultation_type,
        session_duration_minutes: payload.specialist.session_duration_minutes,
        primary_phone: payload.specialist.primary_phone.as_deref(),
        secondary_phone: payload.specialist.secondary_phone.as_deref(),
        languages_spoken: payload.specialist.languages_spoken.as_deref(),
        country: payload.specialist.country.as_deref(),
        time_zone: payload.specialist.time_zone.as_deref(),
        license_url: payload.specialist.license_url.as_deref(),
        suspended: payload.specialist.suspended,
    };

    // Update main specialist fields
    let updated = diesel::update(
        specialists::table
            .filter(specialists::id.eq(specialist_id))
            .filter(specialists::user_id.eq(user.id)),
    )
    .set(&specialist_changeset)
    .returning(Specialist::as_select())
    .get_result::<Specialist>(conn)
    .optional()?
    .ok_or_else(|| AppError::NotFound("Specialist not found".into()))?;

    // Convert payload to user changeset
    let user_changeset = UserChangeset {
        first_name: payload.specialist.first_name.as_deref(),
        last_name: payload.specialist.last_name.as_deref(),
        address: payload.specialist.address.as_deref(),
        city: payload.specialist.city.as_deref(),
        state: payload.specialist.state.as_deref(),
        country: payload.specialist.country.as_deref(),
    };

    // Update user fields
    diesel::update(users::table.filter(users::id.eq(user.id)))
        .set(&user_changeset)
        .execute(conn)?;

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
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSpecialtyRequest {
    pub name: String,
    pub description: Option<String>,
}

pub fn add_specialty(
    conn: &mut PgConnection,
    payload: CreateSpecialtyRequest,
) -> Result<crate::models::Specialty, AppError> {
    use crate::schema::specialties::dsl::*;

    let new_specialty = diesel::insert_into(specialties)
        .values((name.eq(payload.name), description.eq(payload.description)))
        .get_result::<crate::models::Specialty>(conn)?;

    Ok(new_specialty)
}

pub fn list_specialties(
    conn: &mut PgConnection,
) -> Result<Vec<crate::models::Specialty>, AppError> {
    use crate::schema::specialties::dsl::*;

    let results = specialties
        .order_by(name.asc())
        .load::<crate::models::Specialty>(conn)?;

    Ok(results)
}

pub fn upload_license(
    conn: &mut PgConnection,
    _user_id: Uuid,
    _user: &User,
    url: &str,
) -> Result<(), AppError> {
    use crate::schema::specialists::dsl::*;

    diesel::update(specialists.filter(user_id.eq(user_id)))
        .set(license_url.eq(url))
        .execute(conn)?;

    Ok(())
}
