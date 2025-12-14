use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use diesel::prelude::*;
use crate::schema::{
    users, 
    password_reset_tokens,
};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<String>,
    pub dob: Option<NaiveDate>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    pub phone: Option<&'a str>,
    pub gender: Option<&'a str>,
    pub dob: Option<NaiveDate>,
    pub password_hash: &'a str,
    pub role: &'a str,
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



/* Patient */
#[derive(Debug, Serialize, Deserialize)]
pub struct Patient {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub dob: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub blood_type: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub medical_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePatient {
    pub first_name: String,
    pub last_name: String,
    pub dob: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub blood_type: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub medical_notes: Option<String>,
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
