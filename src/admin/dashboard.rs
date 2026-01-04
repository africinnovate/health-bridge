use diesel::prelude::*;
use chrono::{DateTime, Utc};
use chrono::Datelike;


use crate::admin::dtos::{
    AdminActivity, AdminDashboardResponse, AdminUserResponse, PaginationMeta, StatCard, UserFilters, UserListResponse
};
use crate::models::User;
use crate::{
    error::AppError, 
    schema::{
        blood_requests, hospitals, specialists, users
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

pub fn get_users_paginated(
    conn: &mut PgConnection,
    filters: UserFilters,
) -> Result<UserListResponse, AppError> {
    use crate::schema::users::dsl::*;

    let mut count_query = users.into_boxed();
    let mut data_query = users.into_boxed();

    // Apply role filter
    if let Some(filter_role) = filters.role {
        count_query = count_query.filter(role.eq(filter_role));
        data_query = data_query.filter(role.eq(filter_role));
    }

    // Apply search filter
    if let Some(search_term) = filters.search {
        let search_pattern = format!("%{}%", search_term);
        count_query = count_query.filter(
            first_name.ilike(search_pattern.clone())
                .or(last_name.ilike(search_pattern.clone()))
                .or(email.ilike(search_pattern.clone()))
                .or(phone.ilike(search_pattern.clone()))
        );
        data_query = data_query.filter(
            first_name.ilike(search_pattern.clone())
                .or(last_name.ilike(search_pattern.clone()))
                .or(email.ilike(search_pattern.clone()))
                .or(phone.ilike(search_pattern.clone()))
        );
    }

    // Get total count before pagination
    let total_items = count_query
        .count()
        .get_result::<i64>(conn)?;

    // Calculate pagination
    let total_pages = (total_items as f64 / filters.page_size as f64).ceil() as i64;
    let offset = (filters.page - 1) * filters.page_size;

    // Apply pagination and ordering
    let user_list = data_query
        .order(created_at.desc())
        .limit(filters.page_size)
        .offset(offset)
        .select(User::as_select())
        .load::<User>(conn)?;

    // Transform to response format
    let data = user_list
        .into_iter()
        .map(|user| {
            // Determine status based on email verification
            let status = if user.email_verified {
                "Active".to_string()
            } else {
                "Pending".to_string()
            };

            AdminUserResponse {
                id: user.id,
                first_name: user.first_name,
                last_name: user.last_name,
                email: user.email,
                phone: user.phone,
                gender: user.gender,
                dob: user.dob,
                role: user.role,
                email_verified: user.email_verified,
                created_at: user.created_at,
                country: Some("Nigeria".to_string()), // You might want to add this to User model
                status,
            }
        })
        .collect();

    Ok(UserListResponse {
        data,
        pagination: PaginationMeta {
            page: filters.page,
            page_size: filters.page_size,
            total_items,
            total_pages,
        },
    })
}

// pub fn get_admin_users(
//     conn: &mut PgConnection,
//     filters: AdminUserFilters,
// ) -> Result<PaginatedResponse<AdminUserResponse>, AppError> {

//     let page = filters.page.unwrap_or(1).max(1);
//     let per_page = filters.per_page.unwrap_or(20).clamp(1, 100);
//     let offset = (page - 1) * per_page;

//     // Build base query conditions
//     let mut count_query = users::table.into_boxed();
//     let mut data_query = users::table.into_boxed();

//     // 🔎 Role filter (drives tabs)
//     if let Some(role) = &filters.role {
//         count_query = count_query.filter(users::role.eq(role));
//         data_query = data_query.filter(users::role.eq(role));
//     }

//     let total: i64 = count_query
//         .count()
//         .get_result(conn)?;

//     let rows = data_query
//         .order(users::created_at.desc())
//         .limit(per_page)
//         .offset(offset)
//         .load::<crate::models::User>(conn)?;

//     let user_ids: Vec<Uuid> = rows.iter().map(|u| u.id).collect();

//     // 🔗 Batch-load related tables (no N+1)
//     let patient_map = patients::table
//         .filter(patients::user_id.eq_any(&user_ids))
//         .load::<crate::models::Patient>(conn)?
//         .into_iter()
//         .map(|p| (p.user_id, p))
//         .collect::<std::collections::HashMap<_, _>>();

//     let specialist_map = specialists::table
//         .filter(specialists::user_id.eq_any(&user_ids))
//         .load::<crate::models::Specialist>(conn)?
//         .into_iter()
//         .map(|s| (s.user_id, s))
//         .collect::<std::collections::HashMap<_, _>>();

//     let hospital_map = hospitals::table
//         .filter(hospitals::user_id.eq_any(&user_ids))
//         .select(crate::models::Hospital::as_select())
// .load::<crate::models::Hospital>(conn)?
//         .into_iter()
//         .map(|h| (h.user_id, h))
//         .collect::<std::collections::HashMap<_, _>>();

//     let data = rows
//         .into_iter()
//         .map(|u| AdminUserResponse {
//             id: u.id,
//             first_name: u.first_name,
//             last_name: u.last_name,
//             email: u.email,
//             phone: u.phone,
//             role: u.role.to_string(),
//             email_verified: u.email_verified,
//             created_at: u.created_at,

//             patient: patient_map.get(&u.id).map(|p| PatientMeta {
//                 blood_type: p.blood_type.clone(),
//             }),

//             specialist: specialist_map.get(&u.id).map(|s| SpecialistMeta {
//                 specialty_id: s.specialty_id,
//                 verified: s.verified,
//                 suspended: s.suspended,
//             }),

//             hospital: hospital_map.get(&u.id).map(|h| HospitalMeta {
//                 name: h.name.clone(),
//                 city: h.city.clone(),
//                 license_status: h.license_status,
//             }),
//         })
//         .collect();

//     Ok(PaginatedResponse {
//         data,
//         page,
//         per_page,
//         total,
//     })
// }