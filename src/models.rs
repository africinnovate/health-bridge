use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
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
