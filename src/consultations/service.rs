use diesel::prelude::*;
use uuid::Uuid;
use crate::models::{
    ConsultationType, NewConsultationType, UpdateConsultationType,
    ConsultationBenefit, NewConsultationBenefit, UpdateConsultationBenefit,
    ConsultationTypeBenefit,
};
use crate::schema::{consultation_types, consultation_benefits, consultation_type_benefits};
use crate::error::AppError;

// ---- Consultation types CRUD ----

pub fn list_consultation_types(conn: &mut PgConnection) -> Result<Vec<ConsultationType>, AppError> {
    let types = consultation_types::table.load::<ConsultationType>(conn)?;
    Ok(types)
}

pub fn create_consultation_type(
    conn: &mut PgConnection,
    payload: NewConsultationType,
) -> Result<ConsultationType, AppError> {
    let consultation_type = diesel::insert_into(consultation_types::table)
        .values(&payload)
        .get_result::<ConsultationType>(conn)?;
    Ok(consultation_type)
}

pub fn update_consultation_type(
    conn: &mut PgConnection,
    id: Uuid,
    payload: UpdateConsultationType,
) -> Result<ConsultationType, AppError> {
    let consultation_type = diesel::update(consultation_types::table.find(id))
        .set(&payload)
        .get_result::<ConsultationType>(conn)?;
    Ok(consultation_type)
}

pub fn delete_consultation_type(conn: &mut PgConnection, id: Uuid) -> Result<(), AppError> {
    diesel::delete(consultation_types::table.find(id)).execute(conn)?;
    Ok(())
}

// ---- Consultation benefits CRUD ----

pub fn list_consultation_benefits(conn: &mut PgConnection) -> Result<Vec<ConsultationBenefit>, AppError> {
    let benefits = consultation_benefits::table.load::<ConsultationBenefit>(conn)?;
    Ok(benefits)
}

pub fn list_type_benefits(conn: &mut PgConnection, type_id: Uuid) -> Result<Vec<ConsultationBenefit>, AppError> {
    let benefits = consultation_type_benefits::table
        .filter(consultation_type_benefits::consultation_type_id.eq(type_id))
        .inner_join(consultation_benefits::table)
        .select(consultation_benefits::all_columns)
        .load::<ConsultationBenefit>(conn)?;
    Ok(benefits)
}

pub fn create_consultation_benefit(
    conn: &mut PgConnection,
    payload: NewConsultationBenefit,
) -> Result<ConsultationBenefit, AppError> {
    let benefit = diesel::insert_into(consultation_benefits::table)
        .values(&payload)
        .get_result::<ConsultationBenefit>(conn)?;
    Ok(benefit)
}

pub fn update_consultation_benefit(
    conn: &mut PgConnection,
    id: Uuid,
    payload: UpdateConsultationBenefit,
) -> Result<ConsultationBenefit, AppError> {
    let benefit = diesel::update(consultation_benefits::table.find(id))
        .set(&payload)
        .get_result::<ConsultationBenefit>(conn)?;
    Ok(benefit)
}

pub fn delete_consultation_benefit(conn: &mut PgConnection, id: Uuid) -> Result<(), AppError> {
    diesel::delete(consultation_benefits::table.find(id)).execute(conn)?;
    Ok(())
}

// ---- Type-Benefit Linking ----

pub fn link_benefit_to_type(
    conn: &mut PgConnection,
    type_id: Uuid,
    benefit_id: Uuid,
) -> Result<ConsultationTypeBenefit, AppError> {
    let join_record = diesel::insert_into(consultation_type_benefits::table)
        .values(ConsultationTypeBenefit {
            consultation_type_id: type_id,
            consultation_benefit_id: benefit_id,
        })
        .get_result::<ConsultationTypeBenefit>(conn)?;
    Ok(join_record)
}

pub fn unlink_benefit_from_type(
    conn: &mut PgConnection,
    type_id: Uuid,
    benefit_id: Uuid,
) -> Result<(), AppError> {
    diesel::delete(
        consultation_type_benefits::table
            .filter(consultation_type_benefits::consultation_type_id.eq(type_id))
            .filter(consultation_type_benefits::consultation_benefit_id.eq(benefit_id)),
    )
    .execute(conn)?;
    Ok(())
}
