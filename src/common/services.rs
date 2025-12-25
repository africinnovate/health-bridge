use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    models::{Patient, MedicalInfo},
    schema::patients::dsl::*,
    error::AppError,
};

pub fn upsert_medical_info(
    conn: &mut PgConnection,
    user_id_: Uuid,
    data: MedicalInfo<'_>,
) -> Result<Patient, AppError> {
    // Check if patient record exists
    let existing = patients
        .filter(user_id.eq(user_id_))
        .first::<Patient>(conn)
        .optional()?;

    let result = match existing {
        Some(_) => {
            diesel::update(patients.filter(user_id.eq(user_id_)))
                .set((
                    blood_type.eq(data.blood_type),
                    chronic_illnesses.eq(data.chronic_illnesses),
                    allergies.eq(data.allergies),
                    hmo_number.eq(data.hmo_number),
                    emergency_contact_name.eq(data.emergency_contact_name),
                    emergency_contact_phone.eq(data.emergency_contact_phone),
                    medical_notes.eq(data.medical_notes),
                ))
                .get_result::<Patient>(conn)?
        }
        None => {
            diesel::insert_into(patients)
                .values(&data)
                .get_result::<Patient>(conn)?
        }
    };

    Ok(result)
}
