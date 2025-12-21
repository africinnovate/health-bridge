use diesel::prelude::*;
use uuid::Uuid;
use crate::{
    models::{User, Patient, NewPatient, PatientProfile},
    schema::{users, patients},
    error::AppError,
};

pub fn create_patient(
    conn: &mut PgConnection,
    user_id: Uuid,
    blood_type: Option<&str>,
    chronic_illnesses: Option<&str>,
    allergies: Option<&str>,
    hmo_number: Option<&str>,
    emergency_contact_name: Option<&str>,
    emergency_contact_phone: Option<&str>,
    medical_notes: Option<&str>,
) -> Result<Patient, AppError> {
    let new_patient = NewPatient {
        user_id,
        blood_type,
        chronic_illnesses,
        allergies,
        hmo_number,
        emergency_contact_name,
        emergency_contact_phone,
        medical_notes,
    };

    diesel::insert_into(patients::table)
        .values(&new_patient)
        .get_result(conn)
        .map_err(AppError::from)
}

pub fn get_patient_profile(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<PatientProfile, AppError> {
    let user = users::table
        .filter(users::id.eq(user_id))
        .first::<User>(conn)?;

    let patient = patients::table
        .filter(patients::user_id.eq(user_id))
        .first::<Patient>(conn)?;

    Ok(PatientProfile::from_user_and_patient(user, patient))
}

pub fn update_patient(
    conn: &mut PgConnection,
    user_id: Uuid,
    blood_type: Option<&str>,
    chronic_illnesses: Option<&str>,
    allergies: Option<&str>,
    hmo_number: Option<&str>,
    emergency_contact_name: Option<&str>,
    emergency_contact_phone: Option<&str>,
    medical_notes: Option<&str>,
) -> Result<Patient, AppError> {
    diesel::update(patients::table.filter(patients::user_id.eq(user_id)))
        .set((
            patients::blood_type.eq(blood_type),
            patients::chronic_illnesses.eq(chronic_illnesses),
            patients::allergies.eq(allergies),
            patients::hmo_number.eq(hmo_number),
            patients::emergency_contact_name.eq(emergency_contact_name),
            patients::emergency_contact_phone.eq(emergency_contact_phone),
            patients::medical_notes.eq(medical_notes),
        ))
        .get_result(conn)
        .map_err(AppError::from)
}

pub fn list_patients(
    conn: &mut PgConnection,
    limit: i64,
    offset: i64,
) -> Result<Vec<PatientProfile>, AppError> {
    let results = users::table
        .inner_join(patients::table)
        .select((User::as_select(), Patient::as_select()))
        .limit(limit)
        .offset(offset)
        .load::<(User, Patient)>(conn)?;

    Ok(results
        .into_iter()
        .map(|(user, patient)| PatientProfile::from_user_and_patient(user, patient))
        .collect())
}