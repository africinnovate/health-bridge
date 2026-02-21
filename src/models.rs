use crate::{
    schema::{
        admin_audit_logs, appointments, blood_requests, email_verification_tokens,
        hospital_blood_inventories, hospital_settings, hospitals, notifications,
        password_reset_tokens, patients, refresh_tokens, social_accounts,
        specialist_availabilities, specialists, specialties, user_settings, users,
    },
    utils::enums::{
        ActionTypeEnum, AppointmentStatusEnum, AppointmentTypeEnum, BloodTypeEnum, CancelledByEnum,
        ConsultationTypeEnum, DaysOfWeekEnum, Gender, HospitalTypeEnum, NotificationCategoryEnum,
        RequestStatusTypeEnum, Role, TimelineTypeEnum, UrgencyTypeEnum,
    },
};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub image_url: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub dob: Option<NaiveDate>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub email_verified: bool,
    pub note: Option<String>,
    pub eligible_to_donate: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub email: &'a str,
    pub phone: Option<&'a str>,
    pub gender: Option<Gender>,
    pub image_url: Option<&'a str>,
    pub address: Option<&'a str>,
    pub dob: Option<NaiveDate>,
    pub password_hash: &'a str,
    pub role: Role,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub phone: Option<&'a str>,
    pub gender: Option<Gender>,
    pub address: Option<&'a str>,
    pub city: Option<&'a str>,
    pub state: Option<&'a str>,
    pub country: Option<&'a str>,
    pub dob: Option<NaiveDate>,
    pub image_url: Option<&'a str>,
    pub password_hash: &'a str,
    pub role: Role,
    pub note: Option<Option<&'a str>>,
    pub eligible_to_donate: Option<bool>,
    pub deleted_at: Option<DateTime<Utc>>,
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
    pub medications: Option<String>,
    pub existing_conditions: Option<String>,
    pub primary_physician: Option<Uuid>,
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
    pub medications: Option<&'a str>,
    pub existing_conditions: Option<&'a str>,
    pub primary_physician: Option<Uuid>,
    pub hmo_number: Option<&'a str>,
    pub emergency_contact_name: Option<&'a str>,
    pub emergency_contact_phone: Option<&'a str>,
    pub medical_notes: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = hospitals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Hospital {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub hospital_type: Option<HospitalTypeEnum>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub primary_phone: String,
    pub emergency_phone: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub license_number: String,
    pub license_status: bool,
    pub accreditation_doc_url: String,
    pub has_blood_bank: bool,
    pub accepting_donors: bool,
    pub donating_operating_hours: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations, ToSchema,
)]
#[diesel(table_name = hospital_blood_inventories)]
#[diesel(belongs_to(Hospital))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HospitalBloodInventory {
    pub id: Uuid,
    pub hospital_id: Uuid,
    pub blood_type: BloodTypeEnum,
    pub units_available: i32,
    pub bank_capacity: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = hospital_blood_inventories)]
pub struct NewHospitalBloodInventory {
    pub hospital_id: Uuid,
    pub blood_type: BloodTypeEnum,
    pub units_available: i32,
    pub bank_capacity: i32,
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

#[derive(
    Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations, ToSchema,
)]
#[diesel(table_name = appointments)]
#[diesel(belongs_to(BloodRequest))]
#[diesel(belongs_to(Hospital))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Specialist))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Appointment {
    pub id: Uuid,
    pub blood_request_id: Uuid,
    pub hospital_id: Uuid,
    pub user_id: Uuid,
    pub specialist_id: Uuid,

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
#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = specialties)]
pub struct Specialty {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Specialty))]
#[diesel(table_name = specialists)]
pub struct Specialist {
    pub id: Uuid,
    pub user_id: Uuid,
    pub hospital_id: Option<Uuid>,
    pub specialty_id: Uuid,
    pub bio: Option<String>,
    pub years_of_experience: Option<i32>,
    pub consultation_type: ConsultationTypeEnum,
    pub session_duration_minutes: Option<i32>,
    pub primary_phone: Option<String>,
    pub secondary_phone: Option<String>,
    pub languages_spoken: Option<String>,
    pub country: Option<String>,
    pub time_zone: Option<String>,
    pub license_url: Option<String>,
    pub verified: bool,
    pub suspended: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(belongs_to(Specialist))]
#[diesel(table_name = specialist_availabilities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpecialistAvailability {
    pub id: Uuid,
    pub specialist_id: Uuid,
    pub day_of_week: DaysOfWeekEnum,
    pub opens_at: chrono::NaiveTime,
    pub closes_at: chrono::NaiveTime,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = admin_audit_logs)]
pub struct NewAuditLog {
    pub admin_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub action_type: ActionTypeEnum,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, ToSchema)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category: NotificationCategoryEnum,
    pub title: String,
    pub message: String,
    pub related_id: Option<Uuid>,
    pub related_type: Option<String>,
    pub metadata: Option<String>,
    pub is_read: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = notifications)]
pub struct NewNotification {
    pub user_id: Uuid,
    pub category: NotificationCategoryEnum,
    pub title: String,
    pub message: String,
    pub related_id: Option<Uuid>,
    pub related_type: Option<String>,
    pub metadata: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = social_accounts)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SocialAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = social_accounts)]
pub struct NewSocialAccount<'a> {
    pub user_id: Uuid,
    pub provider: &'a str,
    pub provider_user_id: &'a str,
    pub email: Option<&'a str>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    ToSchema,
)]
#[diesel(table_name = user_settings)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserSettings {
    pub id: Uuid,
    pub user_id: Uuid,

    pub appointment_reminders: bool,
    pub specialist_recommendations: bool,
    pub donation_alerts: bool,
    pub account_notifications: bool,

    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub push_notifications: bool,

    pub medical_profile_visibility: String,
    pub allow_specialists_view_history: bool,
    pub allow_app_analytics: bool,
    pub allow_marketing_notifications: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = user_settings)]
pub struct NewUserSettings {
    pub user_id: Uuid,
}

#[derive(
    Debug, Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, ToSchema,
)]
#[diesel(table_name = hospital_settings)]
#[diesel(belongs_to(Hospital))]
pub struct HospitalSettings {
    pub id: Uuid,
    pub hospital_id: Uuid,

    pub donation_requests: bool,

    pub new_donor_appointments: bool,
    pub donor_appointment_reminders: bool,

    pub login_alerts: bool,
    pub account_notifications: bool,

    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub push_notifications: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = hospital_settings)]
pub struct NewHospitalSettings {
    pub hospital_id: Uuid,
}

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = refresh_tokens)]
#[diesel(belongs_to(User))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = refresh_tokens)]
pub struct NewRefreshToken<'a> {
    pub user_id: Uuid,
    pub token: &'a str,
    pub expires_at: DateTime<Utc>,
}
