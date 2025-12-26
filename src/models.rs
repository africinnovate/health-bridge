use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::{DateTime, Utc, NaiveDate};
use diesel::prelude::*;
use crate::{
    schema::{
        appointments, 
        blood_requests, 
        email_verification_tokens, 
        hospitals, 
        password_reset_tokens, 
        patients, 
        users,
    },
    utils::enums::{
        AppointmentStatusEnum, 
        AppointmentTypeEnum, 
        BloodTypeEnum, 
        CancelledByEnum, 
        Gender, 
        HospitalTypeEnum, 
        RequestStatusTypeEnum, 
        Role, 
        TimelineTypeEnum, 
        UrgencyTypeEnum,
    }
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
pub struct MedicalInfo<'a> {
    pub user_id: Uuid,
    pub blood_type: Option<&'a str>,
    pub chronic_illnesses: Option<&'a str>,
    pub allergies: Option<&'a str>,
    pub hmo_number: Option<&'a str>,
    pub emergency_contact_name: Option<&'a str>,
    pub emergency_contact_phone: Option<&'a str>,
    pub medical_notes: Option<&'a str>,
}


#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = hospitals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Hospital {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub country: String,
    pub primary_phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub license_number: String,
    pub license_status: bool,
    pub accreditation_doc_url: String,
    pub has_blood_bank: bool,
    pub accepting_donors: bool,
    pub donating_operating_hours: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = blood_requests)]
#[diesel(belongs_to(Hospital, foreign_key = hospital_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BloodRequest {
    pub id: Uuid,
    pub hospital_id: Uuid,
    pub donor_id: Option<Uuid>,
    pub recipient_id: Option<Uuid>,
    pub ref_id: String,
    pub units: Option<i32>,
    pub blood_type: Option<BloodTypeEnum>,
    pub urgency: Option<UrgencyTypeEnum>,
    pub timeline_status: Option<TimelineTypeEnum>,
    pub request_status: Option<RequestStatusTypeEnum>,
    pub request_reason: Option<String>,
    pub note: Option<String>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_reason: Option<String>,
    pub preferred_time: Option<DateTime<Utc>>,
    pub donated_at: Option<DateTime<Utc>>,
    pub administered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations, ToSchema)]
#[diesel(table_name = appointments)]
#[diesel(belongs_to(BloodRequest))]
#[diesel(belongs_to(Hospital))]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Appointment {
    pub id: Uuid,
    pub blood_request_id: Uuid,
    pub hospital_id: Uuid,
    pub user_id: Uuid,

    pub appointment_type: AppointmentTypeEnum,
    pub status: AppointmentStatusEnum,

    pub scheduled_time: DateTime<Utc>,
    pub previous_time: Option<DateTime<Utc>>,

    pub cancelled_by: Option<CancelledByEnum>,
    pub cancelled_by_id: Option<Uuid>,
    pub cancelled_reason: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
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

