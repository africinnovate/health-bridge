use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::admin::dtos::{HospitalActionRequest, SpecialistActionRequest};
use crate::error::AppError;
use crate::models::{Hospital, NewAuditLog, Specialist, User};
use crate::utils::enums::ConsultationTypeEnum;

#[derive(Debug, Serialize, ToSchema)]
pub struct SpecialistActionResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub specialty_id: Uuid,
    pub verified: bool,
    pub suspended: bool,
    pub consultation_type: ConsultationTypeEnum,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HospitalActionResponse {
    pub id: Uuid,
    pub name: String,
    pub license_status: bool,
    pub license_number: String,
    pub updated_at: DateTime<Utc>,
}

pub fn update_specialist_status_with_audit(
    conn: &mut PgConnection,
    specialist_id: Uuid,
    admin: &User,
    payload: SpecialistActionRequest,
) -> Result<SpecialistActionResponse, AppError> {
    use crate::schema::specialists::dsl::*;

    // First check if specialist exists and get current state
    let existing_specialist = specialists
        .find(specialist_id)
        .select(Specialist::as_select())
        .first::<Specialist>(conn)
        .map_err(|_| AppError::NotFound("Specialist not found".into()))?;

    let mut action_descriptions = Vec::new();

    // Track what changed for audit log
    if let Some(verified_status) = payload.verified {
        if verified_status != existing_specialist.verified {
            action_descriptions.push(if verified_status {
                "specialist_verified"
            } else {
                "specialist_unverified"
            });
        }
    }
    
    if let Some(suspended_status) = payload.suspended {
        if suspended_status != existing_specialist.suspended {
            action_descriptions.push(if suspended_status {
                "specialist_suspended"
            } else {
                "specialist_unsuspended"
            });
        }
    }

    if action_descriptions.is_empty() {
        return Err(AppError::BadRequest(
            "No changes detected".into(),
        ));
    }

    // Update the specialist
    let updated_specialist = conn.transaction(|conn| {
        let updated = if let Some(verified_status) = payload.verified {
            if let Some(suspended_status) = payload.suspended {
                diesel::update(specialists.find(specialist_id))
                    .set((verified.eq(verified_status), suspended.eq(suspended_status)))
                    .get_result::<Specialist>(conn)?
            } else {
                diesel::update(specialists.find(specialist_id))
                    .set(verified.eq(verified_status))
                    .get_result::<Specialist>(conn)?
            }
        } else if let Some(suspended_status) = payload.suspended {
            diesel::update(specialists.find(specialist_id))
                .set(suspended.eq(suspended_status))
                .get_result::<Specialist>(conn)?
        } else {
            return Err(diesel::result::Error::RollbackTransaction);
        };

        // Log each action to audit log
        for action in action_descriptions {
            log_admin_action(
                conn,
                admin.id,
                "specialist",
                specialist_id,
                action,
                payload.reason.as_deref(),
            )?;
        }

        Ok(updated)
    })?;

    Ok(SpecialistActionResponse {
        id: updated_specialist.id,
        user_id: updated_specialist.user_id,
        specialty_id: updated_specialist.specialty_id,
        verified: updated_specialist.verified,
        suspended: updated_specialist.suspended,
        consultation_type: updated_specialist.consultation_type,
        updated_at: Utc::now(),
    })
}

pub fn update_hospital_status_with_audit(
    conn: &mut PgConnection,
    hospital_id: Uuid,
    admin: &User,
    payload: HospitalActionRequest,
) -> Result<HospitalActionResponse, AppError> {
    use crate::schema::hospitals::dsl::*;

    // First check if hospital exists and get current state
    let existing_hospital = hospitals
        .find(hospital_id)
        .select(Hospital::as_select())
        .first::<Hospital>(conn)
        .map_err(|_| AppError::NotFound("Hospital not found".into()))?;

    if payload.license_status.is_none() {
        return Err(AppError::BadRequest(
            "license_status field must be provided".into(),
        ));
    }

    let new_status = payload.license_status.unwrap();
    
    if new_status == existing_hospital.license_status {
        return Err(AppError::BadRequest(
            "No changes detected".into(),
        ));
    }

    let action = if new_status {
        "hospital_approved"
    } else {
        "hospital_revoked"
    };

    let updated_hospital = conn.transaction::<Hospital, diesel::result::Error, _>(|conn| {
    let updated = diesel::update(hospitals.find(hospital_id))
        .set(license_status.eq(new_status))
        .returning(Hospital::as_returning())
        .get_result(conn)?;

    log_admin_action(
        conn,
        admin.id,
        "hospital",
        hospital_id,
        action,
        payload.reason.as_deref(),
    )?;

    Ok(updated)
  })?;


    Ok(HospitalActionResponse {
        id: updated_hospital.id,
        name: updated_hospital.name,
        license_status: updated_hospital.license_status,
        license_number: updated_hospital.license_number,
        updated_at: Utc::now(),
    })
}

// Helper function to log admin actions
fn log_admin_action(
    conn: &mut PgConnection,
    admin_id: Uuid,
    target_type: &str,
    target_id: Uuid,
    action_str: &str,
    reason: Option<&str>,
) -> Result<(), diesel::result::Error> {
    use crate::schema::admin_audit_logs;
    use crate::utils::enums::ActionTypeEnum;
    
    // Convert string to enum
    let action_type = match action_str {
        "specialist_verified" => ActionTypeEnum::SpecialistVerified,
        "specialist_unverified" => ActionTypeEnum::SpecialistUnverified,
        "specialist_suspended" => ActionTypeEnum::SpecialistSuspended,
        "specialist_unsuspended" => ActionTypeEnum::SpecialistUnsuspended,
        "hospital_approved" => ActionTypeEnum::HospitalApproved,
        "hospital_revoked" => ActionTypeEnum::HospitalRevoked,
        _ => return Err(diesel::result::Error::NotFound),
    };
    
    let new_log = NewAuditLog {
        admin_id,
        target_type: target_type.to_string(),
        target_id,
        action_type,
        reason: reason.map(|r| r.to_string()),
    };
    
    diesel::insert_into(admin_audit_logs::table)
        .values(&new_log)
        .execute(conn)?;
    
    Ok(())
}