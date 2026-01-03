use diesel::prelude::*;
use chrono::{DateTime, Utc};
use chrono::Datelike;


use crate::{
    admin::dtos::{AdminActivity, AdminDashboardResponse, StatCard}, 
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

    use diesel::dsl::*;

    let nw = chrono::Utc::now();
    let start_of_month = nw.date_naive().with_day(1).unwrap().and_hms_opt(0,0,0).unwrap();

    // ======================
    // Counts
    // ======================
    let total_users = users::table.count().get_result::<i64>(conn)?;
    let users_this_month = users::table
        .filter(users::created_at.ge(start_of_month))
        .count()
        .get_result::<i64>(conn)?;

    let total_specialists = specialists::table.count().get_result::<i64>(conn)?;
    let specialists_this_month = specialists::table
        .filter(specialists::created_at.ge(start_of_month))
        .count()
        .get_result::<i64>(conn)?;

    let total_hospitals = hospitals::table.count().get_result::<i64>(conn)?;
    let hospitals_this_month = hospitals::table
        .filter(hospitals::created_at.ge(start_of_month))
        .count()
        .get_result::<i64>(conn)?;

    let active_blood_requests = blood_requests::table
        .filter(blood_requests::request_status.is_not_null())
        .count()
        .get_result::<i64>(conn)?;

    let urgent_blood_requests = blood_requests::table
        .filter(blood_requests::urgency.is_not_null())
        .count()
        .get_result::<i64>(conn)?;

    let pending_verifications = specialists::table
        .filter(specialists::verified.eq(false))
        .count()
        .get_result::<i64>(conn)?;

    let total_donation_units = blood_requests::table
        .select(sum(blood_requests::units))
        .first::<Option<i64>>(conn)?
        .unwrap_or(0);

    let total_donors = blood_requests::table
        .filter(blood_requests::donor_id.is_not_null())
        .select(blood_requests::donor_id)
        .distinct()
        .count()
        .get_result::<i64>(conn)?;

    // ======================
    // Percentage helper
    // ======================
    let pct = |this_month: i64, total: i64| -> String {
        if total == 0 {
            "0% this month".into()
        } else {
            format!("+{}% this month", (this_month * 100) / total)
        }
    };

    let stats = vec![
        StatCard {
            label: "Total Users".into(),
            value: total_users,
            sub_label: pct(users_this_month, total_users),
            sub_value: None,
        },
        StatCard {
            label: "Total Specialists".into(),
            value: total_specialists,
            sub_label: pct(specialists_this_month, total_specialists),
            sub_value: None,
        },
        StatCard {
            label: "Total Hospitals".into(),
            value: total_hospitals,
            sub_label: pct(hospitals_this_month, total_hospitals),
            sub_value: None,
        },
        StatCard {
            label: "Active Blood Requests".into(),
            value: active_blood_requests,
            sub_label: "urgent".into(),
            sub_value: Some(urgent_blood_requests.to_string()),
        },
        StatCard {
            label: "Pending Verifications".into(),
            value: pending_verifications,
            sub_label: "Requires action".into(),
            sub_value: None,
        },
        StatCard {
            label: "Total Donation Count".into(),
            value: total_donation_units,
            sub_label: "From donors".into(),
            sub_value: Some(total_donors.to_string()),
        },
    ];

    // ======================
    // Recent activities (unchanged)
    // ======================
    let mut activities = Vec::new();

    activities.extend(
        specialists::table
            .select(specialists::created_at)
            .order(specialists::created_at.desc())
            .limit(3)
            .load::<DateTime<Utc>>(conn)?
            .into_iter()
            .map(|ts| AdminActivity {
                title: "New specialist applied for verification".into(),
                created_at: ts,
            }),
    );

    activities.extend(
        hospitals::table
            .select(hospitals::created_at)
            .order(hospitals::created_at.desc())
            .limit(3)
            .load::<DateTime<Utc>>(conn)?
            .into_iter()
            .map(|ts| AdminActivity {
                title: "New hospital registered in the system".into(),
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
