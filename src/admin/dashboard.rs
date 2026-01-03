use diesel::prelude::*;
use chrono::{DateTime, Utc};

use crate::{
    admin::dtos::{AdminActivity, AdminDashboardResponse, AdminStats}, 
    error::AppError, 
    schema::{
        appointments, blood_requests, hospitals, specialists, users
    }
};



///
/// Admin dashboard read-model
///
pub fn get_admin_dashboard(
    conn: &mut PgConnection,
) -> Result<AdminDashboardResponse, AppError> {

    // ======================
    // Stats
    // ======================
    let total_users = users::table.count().get_result(conn)?;
    let total_specialists = specialists::table.count().get_result(conn)?;
    let total_hospitals = hospitals::table.count().get_result(conn)?;

    let active_blood_requests = blood_requests::table
        .filter(blood_requests::request_status.is_not_null())
        .count()
        .get_result(conn)?;

    let urgent_blood_requests = blood_requests::table
        .filter(blood_requests::urgency.is_not_null())
        .count()
        .get_result(conn)?;

    let pending_verifications = specialists::table
        .filter(specialists::verified.eq(false))
        .count()
        .get_result(conn)?;

    let total_donation_units = blood_requests::table
        .select(diesel::dsl::sum(blood_requests::units))
        .first::<Option<i64>>(conn)?
        .unwrap_or(0);

    let total_donors = blood_requests::table
        .filter(blood_requests::donor_id.is_not_null())
        .select(blood_requests::donor_id)
        .distinct()
        .count()
        .get_result(conn)?;

    let stats = AdminStats {
        total_users,
        total_specialists,
        total_hospitals,
        active_blood_requests,
        urgent_blood_requests,
        pending_verifications,
        total_donation_units,
        total_donors,
    };

    // ======================
    // Recent activity
    // ======================
    let mut activities: Vec<AdminActivity> = Vec::new();

    // Specialists
    let specialist_events = specialists::table
        .select(specialists::created_at)
        .order(specialists::created_at.desc())
        .limit(5)
        .load::<DateTime<Utc>>(conn)?;

    activities.extend(
        specialist_events.into_iter().map(|ts| AdminActivity {
            title: "New specialist applied for verification".into(),
            created_at: ts,
        }),
    );

    // Hospitals
    let hospital_events = hospitals::table
        .select(hospitals::created_at)
        .order(hospitals::created_at.desc())
        .limit(5)
        .load::<DateTime<Utc>>(conn)?;

    activities.extend(
        hospital_events.into_iter().map(|ts| AdminActivity {
            title: "New hospital registered".into(),
            created_at: ts,
        }),
    );

    // Appointments
    let appointment_events = appointments::table
        .select(appointments::created_at)
        .order(appointments::created_at.desc())
        .limit(5)
        .load::<DateTime<Utc>>(conn)?;

    activities.extend(
        appointment_events.into_iter().map(|ts| AdminActivity {
            title: "Appointment updated".into(),
            created_at: ts,
        }),
    );

    activities.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    activities.truncate(5);

    Ok(AdminDashboardResponse {
        stats,
        recent_activities: activities,
    })
}
