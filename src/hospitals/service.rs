use diesel::prelude::*;
use diesel::pg::PgConnection;

use crate::services::mail::MailService;
use crate::{
    error::AppError,
    models::{Hospital, User},
    schema::hospitals::dsl::*,
    utils::enums::Role,
    utils::validation::{validate_email, validate_phone_length},
    handlers::hospitals::{CreateHospitalRequest, UpdateHospitalRequest},
};

/// Create hospital profile (Hospital users only)
pub fn create_hospital(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateHospitalRequest,
    mail_service: &MailService,
) -> Result<Hospital, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized("Only Hospital users can create hospital profile".into()));
    }

    if let Some(ref email_val) = payload.email {
        validate_email(email_val)?;
    }
    validate_phone_length(&payload.primary_phone, 11, 15)?;
    if let Some(ref emergency) = payload.emergency_phone {
        validate_phone_length(emergency, 11, 15)?;
    }

    let hospital: Hospital = diesel::insert_into(hospitals)
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
        .map_err(AppError::from)?;

      if let Some(ref hospital_email) = hospital.email {
        let mail_service = mail_service.clone();
        let hospital_name = hospital.name.clone();
        let to_email = hospital_email.clone();
  

        tokio::spawn(async move {
            let _ = mail_service.send_notification(
                &to_email,
                "Hospital Profile Created",
                &format!("<p>Your hospital <strong>{}</strong> has been successfully created!</p>", hospital_name),
                Some(&format!("Your hospital {} has been successfully created!", hospital_name)),
            ).await;
        });
    }

    Ok(hospital)

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

    if let Some(ref email_val) = payload.email {
        validate_email(email_val)?;
    }
    if let Some(ref phone_val) = payload.primary_phone {
        validate_phone_length(phone_val, 11, 15)?;
    }
    if let Some(ref emergency) = payload.emergency_phone {
        validate_phone_length(emergency, 11, 15)?;
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


