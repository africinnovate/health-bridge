use diesel::pg::PgConnection;
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::services::mail::MailService;
use crate::{
    error::AppError,
    handlers::hospitals::{CreateHospitalRequest, UpdateHospitalRequest},
    models::{Hospital, User},
    schema::hospitals::dsl::*,
    utils::enums::{HospitalTypeEnum, Role},
    utils::validation::{validate_email, validate_phone_length},
};

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::hospitals)]
struct UpdateHospitalChangeset {
    name: Option<String>,
    hospital_type: Option<HospitalTypeEnum>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    primary_phone: Option<String>,
    emergency_phone: Option<String>,
    email: Option<String>,
    accreditation_doc_url: Option<String>,
    has_blood_bank: Option<bool>,
    accepting_donors: Option<bool>,
    donating_operating_hours: Option<String>,
    license_status: Option<bool>,
}

/// Create hospital profile (Hospital users only)
pub fn create_hospital(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateHospitalRequest,
    mail_service: &MailService,
) -> Result<Hospital, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized(
            "Only Hospital users can create hospital profile".into(),
        ));
    }

    if let Some(ref email_val) = payload.email {
        validate_email(email_val)?;
    }
    validate_phone_length(&payload.primary_phone, 11, 15)?;
    if let Some(ref emergency) = payload.emergency_phone {
        validate_phone_length(emergency, 11, 15)?;
    }

    let hospital: Hospital = conn.transaction::<Hospital, AppError, _>(|conn| {
        let hospital: Hospital = diesel::insert_into(hospitals)
            .values((
                user_id.eq(user.id),
                name.eq(payload.name),
                hospital_type.eq(payload.hospital_type),
                address.eq(payload.address),
                city.eq(payload.city),
                state.eq(payload.state),
                country.eq(payload.country),
                primary_phone.eq(payload.primary_phone),
                emergency_phone.eq(payload.emergency_phone),
                email.eq(&payload.email),
                license_number.eq(payload.license_number),
                accreditation_doc_url.eq(payload.accreditation_doc_url),
                has_blood_bank.eq(payload.has_blood_bank),
                accepting_donors.eq(payload.accepting_donors),
                donating_operating_hours.eq(payload.donating_operating_hours),
            ))
            .returning(Hospital::as_select())
            .get_result(conn)
            .map_err(AppError::from)?;

        if let Some(inventory) = payload.blood_inventory {
            use crate::schema::hospital_blood_inventories::dsl::*;
            for item in inventory {
                diesel::insert_into(hospital_blood_inventories)
                    .values((
                        hospital_id.eq(hospital.id),
                        blood_type.eq(item.blood_type),
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .execute(conn)?;
            }
        }

        Ok(hospital)
    })?;

    if let Some(ref hospital_email) = hospital.email {
        let mail_service = mail_service.clone();
        let hospital_name = hospital.name.clone();
        let to_email = hospital_email.clone();

        tokio::spawn(async move {
            let _ = mail_service
                .send_notification(
                    &to_email,
                    "Hospital Profile Created",
                    &format!(
                        "<p>Your hospital <strong>{}</strong> has been successfully created!</p>",
                        hospital_name
                    ),
                    Some(&format!(
                        "Your hospital {} has been successfully created!",
                        hospital_name
                    )),
                )
                .await;
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

    let changeset = UpdateHospitalChangeset {
        name: payload.name,
        hospital_type: payload.hospital_type,
        address: payload.address,
        city: payload.city,
        state: payload.state,
        country: payload.country,
        primary_phone: payload.primary_phone,
        emergency_phone: payload.emergency_phone,
        email: payload.email,
        accreditation_doc_url: payload.accreditation_doc_url,
        has_blood_bank: payload.has_blood_bank,
        accepting_donors: payload.accepting_donors,
        donating_operating_hours: payload.donating_operating_hours,
        license_status: payload.license_status,
    };

    conn.transaction::<Hospital, AppError, _>(|conn| {
        let updated = diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(user_id.eq(user.id)),
        )
        .set(changeset)
        .returning(Hospital::as_select())
        .get_result::<Hospital>(conn)
        .optional()?;

        let hospital_record = match updated {
            Some(h) => h,
            None => return Err(AppError::NotFound("Hospital not found".into())),
        };

        if let Some(inventory) = payload.blood_inventory {
            use crate::schema::hospital_blood_inventories::dsl::*;
            for item in inventory {
                diesel::insert_into(hospital_blood_inventories)
                    .values((
                        hospital_id.eq(hospital_record.id),
                        blood_type.eq(item.blood_type),
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .on_conflict((hospital_id, blood_type))
                    .do_update()
                    .set((
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .execute(conn)?;
            }
        }

        Ok(hospital_record)
    })
}

pub fn delete_hospital(
    conn: &mut PgConnection,
    hospital_id: Uuid,
    user: &User,
) -> Result<(), AppError> {
    use crate::schema::hospitals::dsl::*;
    use crate::utils::enums::Role;
    use chrono::Utc;
    use diesel::prelude::*;

    let affected = if user.role == Role::Admin {
        // Admin can delete any hospital
        diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(deleted_at.is_null()),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(conn)?
    } else {
        // Hospital can only delete their own
        diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(user_id.eq(user.id))
                .filter(deleted_at.is_null()),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(conn)?
    };

    if affected == 0 {
        return Err(AppError::NotFound(
            "Hospital not found or not authorized to delete".into(),
        ));
    }

    Ok(())
}

/// Update hospital accreditation document URL
pub fn update_accreditation_doc(
    conn: &mut PgConnection,
    hospital_id_: Uuid,
    user: &User,
    url: &str,
) -> Result<Hospital, AppError> {
    use crate::schema::hospitals::dsl::*;

    info!(
        "Updating hospital accreditation document URL for hospital {}",
        hospital_id_
    );
    info!(
        "Updating hospital accreditation document URL for user {}",
        user.id
    );

    let updated = diesel::update(
        hospitals
            .filter(id.eq(hospital_id_))
            .filter(user_id.eq(user.id)),
    )
    .set(accreditation_doc_url.eq(url))
    .returning(Hospital::as_select())
    .get_result::<Hospital>(conn)
    .optional()?;

    match updated {
        Some(hospital) => Ok(hospital),
        None => Err(AppError::NotFound(
            "Hospital not found or unauthorized".into(),
        )),
    }
}

/// Get all hospitals
pub fn get_hospitals(conn: &mut PgConnection) -> Result<Vec<Hospital>, AppError> {
    hospitals
        .filter(deleted_at.is_null())
        // .filter(email_verified.eq(true))
        .select(Hospital::as_select())
        .load(conn)
        .map_err(AppError::from)
}

/// Get user's hospitals
pub fn get_user_hospitals(conn: &mut PgConnection, uid: Uuid) -> Result<Vec<Hospital>, AppError> {
    hospitals
        .filter(user_id.eq(uid))
        .filter(deleted_at.is_null())
        .select(Hospital::as_select())
        .load(conn)
        .map_err(AppError::from)
}

/// Get hospital by ID
pub fn get_hospital_by_id(
    conn: &mut PgConnection,
    hospital_id: Uuid,
) -> Result<Hospital, AppError> {
    let result = hospitals
        .filter(id.eq(hospital_id))
        .filter(deleted_at.is_null())
        // .filter(email_verified.eq(true))
        .select(Hospital::as_select())
        .first(conn)
        .optional()?;

    match result {
        Some(hospital) => Ok(hospital),
        None => Err(AppError::NotFound("Hospital not found".into())),
    }
}

pub fn get_hospital_inventory(
    conn: &mut PgConnection,
    h_id: Uuid,
) -> Result<Vec<crate::models::HospitalBloodInventory>, AppError> {
    use crate::schema::hospital_blood_inventories::dsl::*;
    hospital_blood_inventories
        .filter(hospital_id.eq(h_id))
        .load::<crate::models::HospitalBloodInventory>(conn)
        .map_err(AppError::from)
}
