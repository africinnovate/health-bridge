use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use diesel::prelude::*;
use crate::{
    schema::{
        users, 
        patients,
        password_reset_tokens,
        email_verification_tokens,
    },
    utils::enums::{Gender, Role}
};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    pub phone: Option<&'a str>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub password_hash: &'a str,
    pub role: Role,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = email_verification_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = password_reset_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = password_reset_tokens)]
pub struct NewPasswordResetToken<'a> {
    pub user_id: Uuid,
    pub token: &'a str,
    pub expires_at: DateTime<Utc>,
}


#[derive(Insertable)]
#[diesel(table_name = email_verification_tokens)]
pub struct NewEmailVerificationToken<'a> {
    pub user_id: Uuid,
    pub code: &'a str,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}




#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(table_name = patients)]
#[diesel(primary_key(user_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Patient {
    pub user_id: Uuid,
    pub blood_type: Option<String>,
    pub chronic_illnesses: Option<String>,
    pub allergies: Option<String>,
    pub hmo_number: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub medical_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = patients)]
pub struct NewPatient<'a> {
    pub user_id: Uuid,
    pub blood_type: Option<&'a str>,
    pub chronic_illnesses: Option<&'a str>,
    pub allergies: Option<&'a str>,
    pub hmo_number: Option<&'a str>,
    pub emergency_contact_name: Option<&'a str>,
    pub emergency_contact_phone: Option<&'a str>,
    pub medical_notes: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientProfile {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub blood_type: Option<String>,
    pub chronic_illnesses: Option<String>,
    pub allergies: Option<String>,
    pub hmo_number: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub medical_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PatientProfile {
    pub fn from_user_and_patient(user: User, patient: Patient) -> Self {
        Self {
            id: user.id,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            phone: user.phone,
            gender: user.gender,
            dob: user.dob,
            blood_type: patient.blood_type,
            chronic_illnesses: patient.chronic_illnesses,
            allergies: patient.allergies,
            hmo_number: patient.hmo_number,
            emergency_contact_name: patient.emergency_contact_name,
            emergency_contact_phone: patient.emergency_contact_phone,
            medical_notes: patient.medical_notes,
            created_at: user.created_at,
        }
    }
}

/* Hospital */
#[derive(Debug, Serialize, Deserialize)]
pub struct Hospital {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHospital {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
}

/* Specialist */
#[derive(Debug, Serialize, Deserialize)]
pub struct Specialist {
    pub id: Uuid,
    pub hospital_id: Option<Uuid>,
    pub full_name: String,
    pub speciality: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpecialist {
    pub hospital_id: Option<Uuid>,
    pub full_name: String,
    pub speciality: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}
