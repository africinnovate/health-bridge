use diesel::prelude::*;
use chrono::{DateTime, Utc};
use chrono::Datelike;
use uuid::Uuid;


use crate::schema::patients;
use crate::{
    admin::dtos::{
      AdminActivity, 
      AdminDashboardResponse, 
      StatCard,
      AdminUserResponse,
      PaginatedResponse,
      PatientMeta,
      SpecialistMeta,
      HospitalMeta,
      AdminUserFilters
    }, 
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



pub fn get_admin_users(
    conn: &mut PgConnection,
    filters: AdminUserFilters,
) -> Result<PaginatedResponse<AdminUserResponse>, AppError> {

    let page = filters.page.unwrap_or(1).max(1);
    let per_page = filters.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut base = users::table.into_boxed();

    // 🔎 Role filter (drives tabs)
    if let Some(role) = &filters.role {
        base = base.filter(users::role.eq(role));
    }

    let total: i64 = base
        .count()
        .get_result(conn)?;

    let rows = base
        .order(users::created_at.desc())
        .limit(per_page)
        .offset(offset)
        .load::<crate::models::User>(conn)?;

    let user_ids: Vec<Uuid> = rows.iter().map(|u| u.id).collect();

    // 🔗 Batch-load related tables (no N+1)
    let patient_map = patients::table
        .filter(patients::user_id.eq_any(&user_ids))
        .load::<crate::models::Patient>(conn)?
        .into_iter()
        .map(|p| (p.user_id, p))
        .collect::<std::collections::HashMap<_, _>>();

    let specialist_map = specialists::table
        .filter(specialists::user_id.eq_any(&user_ids))
        .load::<crate::models::Specialist>(conn)?
        .into_iter()
        .map(|s| (s.user_id, s))
        .collect::<std::collections::HashMap<_, _>>();

    let hospital_map = hospitals::table
        .filter(hospitals::user_id.eq_any(&user_ids))
        .select(crate::models::Hospital::as_select())
.load::<crate::models::Hospital>(conn)?
        .into_iter()
        .map(|h| (h.user_id, h))
        .collect::<std::collections::HashMap<_, _>>();

    let data = rows
        .into_iter()
        .map(|u| AdminUserResponse {
            id: u.id,
            first_name: u.first_name,
            last_name: u.last_name,
            email: u.email,
            phone: u.phone,
            role: u.role.to_string(),
            email_verified: u.email_verified,
            created_at: u.created_at,

            patient: patient_map.get(&u.id).map(|p| PatientMeta {
                blood_type: p.blood_type.clone(),
            }),

            specialist: specialist_map.get(&u.id).map(|s| SpecialistMeta {
                specialty_id: s.specialty_id,
                verified: s.verified,
                suspended: s.suspended,
            }),

            hospital: hospital_map.get(&u.id).map(|h| HospitalMeta {
                name: h.name.clone(),
                city: h.city.clone(),
                license_status: h.license_status,
            }),
        })
        .collect();

    Ok(PaginatedResponse {
        data,
        page,
        per_page,
        total,
    })
}