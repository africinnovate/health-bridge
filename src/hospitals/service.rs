use diesel::pg::PgConnection;
use diesel::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::services::mail::MailService;
use crate::{
    error::AppError,
    handlers::hospitals::{CreateHospitalRequest, UpdateHospitalRequest},
    models::{Hospital, User},
    schema::hospitals::dsl::*,
    utils::enums::{HospitalTypeEnum, Role},
    utils::validation::{validate_email, validate_phone_length},
};

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::hospitals)]
struct UpdateHospitalChangeset {
    name: Option<String>,
    hospital_type: Option<HospitalTypeEnum>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    primary_phone: Option<String>,
    emergency_phone: Option<String>,
    email: Option<String>,
    accreditation_doc_url: Option<String>,
    has_blood_bank: Option<bool>,
    accepting_donors: Option<bool>,
    donating_operating_hours: Option<String>,
    license_status: Option<bool>,
}

/// Create hospital profile (Hospital users only)
pub fn create_hospital(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateHospitalRequest,
    mail_service: &MailService,
) -> Result<Hospital, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized(
            "Only Hospital users can create hospital profile".into(),
        ));
    }

    if let Some(ref email_val) = payload.email {
        validate_email(email_val)?;
    }
    validate_phone_length(&payload.primary_phone, 11, 15)?;
    if let Some(ref emergency) = payload.emergency_phone {
        validate_phone_length(emergency, 11, 15)?;
    }

    let hospital: Hospital = conn.transaction::<Hospital, AppError, _>(|conn| {
        let hospital: Hospital = diesel::insert_into(hospitals)
            .values((
                user_id.eq(user.id),
                name.eq(payload.name),
                hospital_type.eq(payload.hospital_type),
                address.eq(payload.address),
                city.eq(payload.city),
                state.eq(payload.state),
                country.eq(payload.country),
                primary_phone.eq(payload.primary_phone),
                emergency_phone.eq(payload.emergency_phone),
                email.eq(&payload.email),
                license_number.eq(payload.license_number),
                accreditation_doc_url.eq(payload.accreditation_doc_url),
                has_blood_bank.eq(payload.has_blood_bank),
                accepting_donors.eq(payload.accepting_donors),
                donating_operating_hours.eq(payload.donating_operating_hours),
            ))
            .returning(Hospital::as_select())
            .get_result(conn)
            .map_err(AppError::from)?;

        if let Some(inventory) = payload.blood_inventory {
            use crate::schema::hospital_blood_inventories::dsl::*;
            for item in inventory {
                diesel::insert_into(hospital_blood_inventories)
                    .values((
                        hospital_id.eq(hospital.id),
                        blood_type.eq(item.blood_type),
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .execute(conn)?;
            }
        }

        Ok(hospital)
    })?;

    if let Some(ref hospital_email) = hospital.email {
        let mail_service = mail_service.clone();
        let hospital_name = hospital.name.clone();
        let to_email = hospital_email.clone();

        tokio::spawn(async move {
            let _ = mail_service
                .send_notification(
                    &to_email,
                    "Hospital Profile Created",
                    &format!(
                        "<p>Your hospital <strong>{}</strong> has been successfully created!</p>",
                        hospital_name
                    ),
                    Some(&format!(
                        "Your hospital {} has been successfully created!",
                        hospital_name
                    )),
                )
                .await;
        });
    }

    Ok(hospital)
}

/// Update hospital profile
///
/// - Hospital can update own profile
/// - Only Admin can update `license_status`
/// - Partial updates allowed

pub fn update_hospital(
    conn: &mut PgConnection,
    hospital_id: uuid::Uuid,
    user: &User,
    payload: UpdateHospitalRequest,
) -> Result<Hospital, AppError> {
    // Only admins can update license status
    if payload.license_status.is_some() && user.role != Role::Admin {
        return Err(AppError::Unauthorized(
            "Only admins can update license status".into(),
        ));
    }

    if let Some(ref email_val) = payload.email {
        validate_email(email_val)?;
    }
    if let Some(ref phone_val) = payload.primary_phone {
        validate_phone_length(phone_val, 11, 15)?;
    }
    if let Some(ref emergency) = payload.emergency_phone {
        validate_phone_length(emergency, 11, 15)?;
    }

    let changeset = UpdateHospitalChangeset {
        name: payload.name,
        hospital_type: payload.hospital_type,
        address: payload.address,
        city: payload.city,
        state: payload.state,
        country: payload.country,
        primary_phone: payload.primary_phone,
        emergency_phone: payload.emergency_phone,
        email: payload.email,
        accreditation_doc_url: payload.accreditation_doc_url,
        has_blood_bank: payload.has_blood_bank,
        accepting_donors: payload.accepting_donors,
        donating_operating_hours: payload.donating_operating_hours,
        license_status: payload.license_status,
    };

    conn.transaction::<Hospital, AppError, _>(|conn| {
        let updated = diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(user_id.eq(user.id)),
        )
        .set(changeset)
        .returning(Hospital::as_select())
        .get_result::<Hospital>(conn)
        .optional()?;

        let hospital_record = match updated {
            Some(h) => h,
            None => return Err(AppError::NotFound("Hospital not found".into())),
        };

        if let Some(inventory) = payload.blood_inventory {
            use crate::schema::hospital_blood_inventories::dsl::*;
            for item in inventory {
                diesel::insert_into(hospital_blood_inventories)
                    .values((
                        hospital_id.eq(hospital_record.id),
                        blood_type.eq(item.blood_type),
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .on_conflict((hospital_id, blood_type))
                    .do_update()
                    .set((
                        units_available.eq(item.units_available.unwrap_or(0)),
                        bank_capacity.eq(item.bank_capacity.unwrap_or(0)),
                    ))
                    .execute(conn)?;
            }
        }

        Ok(hospital_record)
    })
}

pub fn delete_hospital(
    conn: &mut PgConnection,
    hospital_id: Uuid,
    user: &User,
) -> Result<(), AppError> {
    use crate::schema::hospitals::dsl::*;
    use crate::utils::enums::Role;
    use chrono::Utc;
    use diesel::prelude::*;

    let affected = if user.role == Role::Admin {
        // Admin can delete any hospital
        diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(deleted_at.is_null()),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(conn)?
    } else {
        // Hospital can only delete their own
        diesel::update(
            hospitals
                .filter(id.eq(hospital_id))
                .filter(user_id.eq(user.id))
                .filter(deleted_at.is_null()),
        )
        .set(deleted_at.eq(Some(Utc::now())))
        .execute(conn)?
    };

    if affected == 0 {
        return Err(AppError::NotFound(
            "Hospital not found or not authorized to delete".into(),
        ));
    }

    Ok(())
}

/// Update hospital accreditation document URL
pub fn update_accreditation_doc(
    conn: &mut PgConnection,
    hospital_id_: Uuid,
    user: &User,
    url: &str,
) -> Result<Hospital, AppError> {
    use crate::schema::hospitals::dsl::*;

    info!(
        "Updating hospital accreditation document URL for hospital {}",
        hospital_id_
    );
    info!(
        "Updating hospital accreditation document URL for user {}",
        user.id
    );

    let updated = diesel::update(
        hospitals
            .filter(id.eq(hospital_id_))
            .filter(user_id.eq(user.id)),
    )
    .set(accreditation_doc_url.eq(url))
    .returning(Hospital::as_select())
    .get_result::<Hospital>(conn)
    .optional()?;

    match updated {
        Some(hospital) => Ok(hospital),
        None => Err(AppError::NotFound(
            "Hospital not found or unauthorized".into(),
        )),
    }
}

/// Get all hospitals
pub fn get_hospitals(conn: &mut PgConnection) -> Result<Vec<Hospital>, AppError> {
    hospitals
        .filter(deleted_at.is_null())
        // .filter(email_verified.eq(true))
        .select(Hospital::as_select())
        .load(conn)
        .map_err(AppError::from)
}

/// Get user's hospitals
pub fn get_user_hospitals(conn: &mut PgConnection, uid: Uuid) -> Result<Vec<Hospital>, AppError> {
    hospitals
        .filter(user_id.eq(uid))
        .filter(deleted_at.is_null())
        .select(Hospital::as_select())
        .load(conn)
        .map_err(AppError::from)
}

/// Get hospital by ID
pub fn get_hospital_by_id(
    conn: &mut PgConnection,
    hospital_id: Uuid,
) -> Result<Hospital, AppError> {
    let result = hospitals
        .filter(id.eq(hospital_id))
        .filter(deleted_at.is_null())
        // .filter(email_verified.eq(true))
        .select(Hospital::as_select())
        .first(conn)
        .optional()?;

    match result {
        Some(hospital) => Ok(hospital),
        None => Err(AppError::NotFound("Hospital not found".into())),
    }
}

/// Get nearby hospitals based on user's city or state
pub fn get_nearby_hospitals(
    conn: &mut PgConnection,
    user: &User,
) -> Result<Vec<Hospital>, AppError> {
    let user_city = user.city.as_deref().unwrap_or("");
    let user_state = user.state.as_deref().unwrap_or("");

    if user_city.is_empty() && user_state.is_empty() {
        return Ok(vec![]);
    }

    let mut query = hospitals.filter(deleted_at.is_null()).into_boxed();

    if !user_city.is_empty() && !user_state.is_empty() {
        query = query.filter(city.eq(user_city).or(state.eq(user_state)));
    } else if !user_city.is_empty() {
        query = query.filter(city.eq(user_city));
    } else {
        query = query.filter(state.eq(user_state));
    }

    query
        .select(Hospital::as_select())
        .load(conn)
        .map_err(AppError::from)
}

pub fn get_hospital_inventory(
    conn: &mut PgConnection,
    h_id: Uuid,
) -> Result<Vec<crate::models::HospitalBloodInventory>, AppError> {
    use crate::schema::hospital_blood_inventories::dsl::*;
    hospital_blood_inventories
        .filter(hospital_id.eq(h_id))
        .load::<crate::models::HospitalBloodInventory>(conn)
        .map_err(AppError::from)
}
pub fn update_blood_inventory(
    conn: &mut PgConnection,
    h_id: Uuid,
    user: &User,
    b_type: crate::utils::enums::BloodTypeEnum,
    payload: crate::handlers::hospitals::UpdateBloodInventoryRequest,
) -> Result<crate::models::HospitalBloodInventory, AppError> {
    use crate::schema::hospital_blood_inventories::dsl::*;
    use crate::schema::hospitals::dsl as h;

    // Check if hospital exists and belongs to user (or user is admin)
    let hospital_exists = h::hospitals
        .filter(h::id.eq(h_id))
        .filter(h::user_id.eq(user.id))
        .select(h::id)
        .first::<Uuid>(conn)
        .optional()?;

    if hospital_exists.is_none() {
        return Err(AppError::NotFound(
            "Hospital not found or unauthorized".into(),
        ));
    }

    // Perform the update
    let result = diesel::update(
        hospital_blood_inventories
            .filter(hospital_id.eq(h_id))
            .filter(blood_type.eq(b_type.clone())),
    )
    .set((
        payload.units_available.map(|v| units_available.eq(v)),
        payload.bank_capacity.map(|v| bank_capacity.eq(v)),
        updated_at.eq(chrono::Utc::now()),
    ))
    .get_result::<crate::models::HospitalBloodInventory>(conn)
    .optional()?;

    match result {
        Some(inventory) => Ok(inventory),
        None => {
            // If it doesn't exist, we could choose to create it or return 404.
            // Given it's an "update" endpoint, 404 seems more appropriate unless we want UPSERT.
            // However, the existing UpdateHospitalRequest does UPSERT. Let's do UPSERT for consistency if needed,
            // but the user asked to "update", and usually inventory is initialized.
            // Let's stick to update for now, but provide a helpful error.
            Err(AppError::NotFound(format!(
                "Inventory for blood type {:?} not found in this hospital",
                b_type
            )))
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct DonorQuery {
    pub eligible_to_donate: Option<bool>,
    pub blood_type: Option<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DonorDetail {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub gender: Option<crate::utils::enums::Gender>,
    pub image_url: Option<String>,
    pub eligible_to_donate: bool,
    pub note: Option<String>,
    pub blood_type: Option<String>,
}

pub fn get_donors(
    conn: &mut PgConnection,
    filters: DonorQuery,
) -> Result<Vec<DonorDetail>, AppError> {
    use crate::schema::{patients::dsl as p, users::dsl as u};
    use crate::utils::enums::Role;

    // We only want users with Role::Donor or Role::Patient
    let mut query = u::users
        .left_join(p::patients.on(p::user_id.eq(u::id)))
        .filter(u::role.eq(Role::Donor).or(u::role.eq(Role::Patient)))
        .filter(u::deleted_at.is_null())
        .into_boxed();

    if let Some(eligible) = filters.eligible_to_donate {
        query = query.filter(u::eligible_to_donate.eq(eligible));
    }

    if let Some(bt) = filters.blood_type {
        query = query.filter(p::blood_type.eq(bt));
    }

    let results = query
        .select((
            u::id,
            u::first_name,
            u::last_name,
            u::email,
            u::phone,
            u::gender,
            u::image_url,
            u::eligible_to_donate,
            u::note,
            p::blood_type.nullable(),
        ))
        .order(u::created_at.desc())
        .load::<(
            Uuid,
            String,
            String,
            String,
            Option<String>,
            Option<crate::utils::enums::Gender>,
            Option<String>,
            bool,
            Option<String>,
            Option<String>,
        )>(conn)?;

    Ok(results
        .into_iter()
        .map(
            |(uid, fname, lname, umail, uphone, ugender, uimg, ueligible, unote, ublood)| {
                DonorDetail {
                    user_id: uid,
                    first_name: fname,
                    last_name: lname,
                    email: umail,
                    phone: uphone,
                    gender: ugender,
                    image_url: uimg,
                    eligible_to_donate: ueligible,
                    note: unote,
                    blood_type: ublood,
                }
            },
        )
        .collect())
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateDonorRequest {
    pub eligible_to_donate: Option<bool>,
    pub note: Option<Option<String>>,
}

pub fn update_donor(
    conn: &mut PgConnection,
    donor_id: Uuid,
    payload: UpdateDonorRequest,
) -> Result<DonorDetail, AppError> {
    use crate::schema::{patients::dsl as p, users::dsl as u};

    let updated_user = diesel::update(u::users.filter(u::id.eq(donor_id)))
        .set(&payload)
        .returning(User::as_select())
        .get_result::<User>(conn)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Donor not found".into()))?;

    let blood_type: Option<String> = p::patients
        .filter(p::user_id.eq(donor_id))
        .select(p::blood_type)
        .first::<Option<String>>(conn)
        .optional()?
        .flatten();

    Ok(DonorDetail {
        user_id: updated_user.id,
        first_name: updated_user.first_name,
        last_name: updated_user.last_name,
        email: updated_user.email,
        phone: updated_user.phone,
        gender: updated_user.gender,
        image_url: updated_user.image_url,
        eligible_to_donate: updated_user.eligible_to_donate,
        note: updated_user.note,
        blood_type,
    })
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct HospitalDashboardStats {
    pub active_blood_requests: i64,
    pub urgent_requests_nearby: i64,
    pub appointments_count: i64,
}

pub fn get_dashboard_stats(
    conn: &mut PgConnection,
    hospital_user_id: Uuid,
) -> Result<HospitalDashboardStats, AppError> {
    use crate::schema::{
        appointments::dsl as appt, blood_requests::dsl as br, hospitals::dsl as h,
    };
    use crate::utils::enums::{AppointmentStatusEnum, RequestStatusTypeEnum, UrgencyTypeEnum};

    // First find the hospital ID associated with this user
    let hospital_id = h::hospitals
        .filter(h::user_id.eq(hospital_user_id))
        .select(h::id)
        .first::<Uuid>(conn)
        .optional()?;

    let (active_requests, appts_count) = if let Some(h_id) = hospital_id {
        let req_count = br::blood_requests
            .filter(br::hospital_id.eq(h_id))
            .filter(br::request_status.ne(RequestStatusTypeEnum::Completed))
            .filter(br::request_status.ne(RequestStatusTypeEnum::Cancelled))
            .count()
            .get_result::<i64>(conn)?;

        let now = chrono::Utc::now();
        let appts = appt::appointments
            .filter(appt::hospital_id.eq(h_id))
            .filter(appt::scheduled_time.gt(now))
            .filter(appt::status.ne(AppointmentStatusEnum::Cancelled))
            .filter(appt::status.ne(AppointmentStatusEnum::Completed))
            .count()
            .get_result::<i64>(conn)?;

        (req_count, appts)
    } else {
        (0, 0)
    };

    let urgent_requests = br::blood_requests
        .filter(br::urgency.eq(UrgencyTypeEnum::Urgent))
        .filter(br::request_status.ne(RequestStatusTypeEnum::Completed))
        .filter(br::request_status.ne(RequestStatusTypeEnum::Cancelled))
        .count()
        .get_result::<i64>(conn)?;

    Ok(HospitalDashboardStats {
        active_blood_requests: active_requests,
        urgent_requests_nearby: urgent_requests,
        appointments_count: appts_count,
    })
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RecentActivityItem {
    pub id: Uuid,
    pub activity_type: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub fn get_recent_activity(
    conn: &mut PgConnection,
    hospital_user_id: Uuid,
) -> Result<Vec<RecentActivityItem>, AppError> {
    use crate::schema::{
        appointments::dsl as appt, blood_requests::dsl as br,
        hospital_blood_inventories::dsl as inv, hospitals::dsl as h,
    };
    use crate::utils::enums::{AppointmentStatusEnum, RequestStatusTypeEnum};

    // First find the hospital ID associated with this user
    let hospital_id = h::hospitals
        .filter(h::user_id.eq(hospital_user_id))
        .select(h::id)
        .first::<Uuid>(conn)
        .optional()?;

    let h_id = match hospital_id {
        Some(hid) => hid,
        None => return Ok(vec![]),
    };

    let mut activities = Vec::new();

    // 1. Blood requests fulfilled or cancelled recently (last 30 days)
    let thirty_days_ago = chrono::Utc::now() - chrono::Duration::days(30);

    let recent_requests = br::blood_requests
        .filter(br::hospital_id.eq(h_id))
        .filter(br::created_at.gt(thirty_days_ago))
        .load::<crate::models::BloodRequest>(conn)?;

    for req in recent_requests {
        if req.request_status == Some(RequestStatusTypeEnum::Completed) {
            activities.push(RecentActivityItem {
                id: req.id,
                activity_type: "BloodRequest".to_string(),
                description: format!("Blood request {} was completed.", req.ref_id),
                timestamp: req.donated_at.unwrap_or(req.created_at),
            });
        } else if req.request_status == Some(RequestStatusTypeEnum::Cancelled) {
            activities.push(RecentActivityItem {
                id: req.id,
                activity_type: "BloodRequest".to_string(),
                description: format!("Blood request {} was cancelled.", req.ref_id),
                timestamp: req.cancelled_at.unwrap_or(req.created_at),
            });
        }
    }

    // 2. Upcoming or recently created appointments
    let recent_appts = appt::appointments
        .filter(appt::hospital_id.eq(h_id))
        .filter(appt::created_at.gt(thirty_days_ago))
        .load::<crate::models::Appointment>(conn)?;

    for appt in recent_appts {
        activities.push(RecentActivityItem {
            id: appt.id,
            activity_type: "Appointment".to_string(),
            description: format!("New appointment scheduled with status {:?}.", appt.status),
            timestamp: appt.created_at,
        });
    }

    // 3. Inventory warnings (low stock)
    let inventory = inv::hospital_blood_inventories
        .filter(inv::hospital_id.eq(h_id))
        .load::<crate::models::HospitalBloodInventory>(conn)?;

    for item in inventory {
        // Warning if less than 20% capacity or less than 5 units total
        let low_stock_threshold = std::cmp::max(5, (item.bank_capacity as f32 * 0.2) as i32);
        if item.units_available <= low_stock_threshold {
            activities.push(RecentActivityItem {
                id: item.id,
                activity_type: "InventoryWarning".to_string(),
                description: format!(
                    "Low stock warning for blood type {:?}: {} units available.",
                    item.blood_type, item.units_available
                ),
                timestamp: item.updated_at,
            });
        }
    }

    // Sort by timestamp descending
    activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Take top 20
    activities.truncate(20);

    Ok(activities)
}
