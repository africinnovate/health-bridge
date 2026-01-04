use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::utils::enums::Role;

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
pub struct AdminUserResponse {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub role: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,

    // Optional extended data
    pub patient: Option<PatientMeta>,
    pub specialist: Option<SpecialistMeta>,
    pub hospital: Option<HospitalMeta>,
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

#[derive(Debug, Deserialize, ToSchema)]
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