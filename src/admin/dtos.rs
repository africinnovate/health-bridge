use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_specialists: i64,
    pub total_hospitals: i64,
    pub active_blood_requests: i64,
    pub urgent_blood_requests: i64,
    pub pending_verifications: i64,
    pub total_donation_units: i64,
    pub total_donors: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminActivity {
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminDashboardResponse {
    pub stats: AdminStats,
    pub recent_activities: Vec<AdminActivity>,
}

