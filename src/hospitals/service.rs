use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::{
    error::AppError,
    models::{Hospital, User},
    schema::hospitals::dsl::*,
    utils::enums::Role,
    handlers::hospitals::{CreateHospitalRequest, UpdateHospitalRequest},
};

/// Create hospital profile (Hospital users only)
pub fn create_hospital(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateHospitalRequest,
) -> Result<Hospital, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized("Only Hospital users can create hospital profile".into()));
    }

    diesel::insert_into(hospitals)
        .values((
            user_id.eq(user.id),
            name.eq(payload.name),
            hospital_type.eq(payload.hospital_type),
            address.eq(payload.address),
            city.eq(payload.city),
            country.eq(payload.country),
            primary_phone.eq(payload.primary_phone),
            emergency_phone.eq(payload.emergency_phone),
            email.eq(payload.email),
            license_number.eq(payload.license_number),
            accreditation_doc_url.eq(payload.accreditation_doc_url),
            has_blood_bank.eq(payload.has_blood_bank),
            accepting_donors.eq(payload.accepting_donors),
            donating_operating_hours.eq(payload.donating_operating_hours),
        ))
        .returning(Hospital::as_select())
        .get_result(conn)
        .map_err(AppError::from)
}


/// Update hospital profile
///
/// - Hospital can update own profile
/// - Only Admin can update `license_status`
/// - Partial updates allowed

pub fn update_hospital(
    conn: &mut PgConnection,
    hospital_id: uuid::Uuid,
    user: &User,
    payload: UpdateHospitalRequest,
) -> Result<Hospital, AppError> {

    // Only admins can update license status
    if payload.license_status.is_some() && user.role != Role::Admin {
        return Err(AppError::Unauthorized(
            "Only admins can update license status".into(),
        ));
    }

    let updated = diesel::update(
        hospitals
            .filter(id.eq(hospital_id))
            .filter(user_id.eq(user.id)),
    )
    .set(payload)
    .returning(Hospital::as_select())
    .get_result::<Hospital>(conn)
    .optional()?;

    match updated {
        Some(hospital) => Ok(hospital),
        None => Err(AppError::NotFound("Hospital not found".into())),
    }
}


