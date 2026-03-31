use crate::utils::enums::{
    AppointmentStatusEnum, ConsultationTypeEnum, Gender, NotificationCategoryEnum,
    RequestStatusTypeEnum, Role, BloodTypeEnum
};
use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct StatCard {
    pub label: String,
    pub value: i64,
    pub sub_label: String,
    pub sub_value: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminActivity {
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminDashboardResponse {
    pub stats: Vec<StatCard>,
    pub recent_activities: Vec<AdminActivity>,
}

#[derive(Serialize)]
pub struct PatientMeta {
    pub blood_type: Option<String>,
}

#[derive(Serialize)]
pub struct SpecialistMeta {
    pub specialty_id: Uuid,
    pub verified: bool,
    pub suspended: bool,
}

#[derive(Serialize)]
pub struct HospitalMeta {
    pub name: String,
    pub city: String,
    pub license_status: bool,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

#[derive(Deserialize)]
pub struct AdminUserFilters {
    pub role: Option<Role>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct UserFilters {
    /// Filter by user role (patient, specialist, hospital, admin)
    pub role: Option<Role>,

    /// Search by name, email, or phone
    pub search: Option<String>,

    /// Page number (starts from 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Number of items per page (default: 10, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub dob: Option<NaiveDate>,
    pub address: Option<String>,
    pub image_url: Option<String>,
    pub role: Role,
    pub email_verified: bool,
    pub consultation_preference: Option<ConsultationTypeEnum>,
    pub created_at: DateTime<Utc>,

    // For patients specifically - could be null for other roles
    pub country: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub data: Vec<AdminUserResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SpecialistActionRequest {
    /// Set to true to verify/approve, false to remove verification
    pub verified: Option<bool>,
    /// Set suspended status (true = suspended, false = active)
    pub suspended: Option<bool>,
    /// Optional reason for the action (required when suspending or removing verification)
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct HospitalActionRequest {
    /// Set license status (true = approved, false = revoked)
    pub license_status: Option<bool>,
    /// Optional reason for the action (required when revoking license)
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct NotificationFilters {
    /// Filter by category (admin only)
    pub category: Option<NotificationCategoryEnum>,

    /// Filter by read status
    pub is_read: Option<bool>,

    /// Search in title or message
    pub search: Option<String>,

    /// Page number (starts from 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Number of items per page (default: 10, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AppointmentHistoryItem {
    pub specialist_name: String,
    pub specialty: String,
    pub scheduled_time: DateTime<Utc>,
    pub status: AppointmentStatusEnum,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DonationHistoryItem {
    pub hospital_name: String,
    pub created_at: DateTime<Utc>,
    pub status: RequestStatusTypeEnum,
    pub blood_type: Option<BloodTypeEnum>,
    pub units: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminPatientProfileResponse {
    pub profile: crate::handlers::patients::PatientProfileResponse,
    pub appointments: Vec<AppointmentHistoryItem>,
    pub donations: Vec<DonationHistoryItem>,
}
