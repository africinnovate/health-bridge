use chrono::{DateTime, Utc};
use diesel::AsChangeset;
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::utils::enums::Role;
use crate::{
    error::AppError,
    models::{BloodRequest, User},
    schema::blood_requests::dsl::*,
    utils::enums::{BloodTypeEnum, RequestStatusTypeEnum, TimelineTypeEnum, UrgencyTypeEnum},
};

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = crate::schema::blood_requests)]
pub struct CreateBloodRequest {
    pub hospital_id: Uuid,
    pub units: Option<i32>,
    pub blood_type: Option<BloodTypeEnum>,
    pub urgency: Option<UrgencyTypeEnum>,
    pub request_reason: Option<String>,
    pub note: Option<String>,
    pub preferred_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BloodRequestQuery {
    pub request_status: Option<RequestStatusTypeEnum>,
    pub blood_type: Option<BloodTypeEnum>,
    pub urgency: Option<UrgencyTypeEnum>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BloodRequestResponse {
    pub blood_request: BloodRequest,
    pub donor: Option<User>,
    pub patient: Option<User>,
}

#[derive(Debug, Deserialize, AsChangeset, Selectable, ToSchema)]
#[diesel(table_name = crate::schema::blood_requests)]
pub struct UpdateBloodRequest {
    pub units: Option<i32>,
    pub urgency: Option<UrgencyTypeEnum>,
    pub timeline_status: Option<TimelineTypeEnum>,
    pub request_status: Option<RequestStatusTypeEnum>,
    pub note: Option<String>,
    pub donated_at: Option<DateTime<Utc>>,
    pub administered_at: Option<DateTime<Utc>>,
    /// Set or clear the donor. Pass `Some(Some(id))` to assign, `Some(None)` to clear.
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::double_option"
    )]
    pub donor_id: Option<Option<Uuid>>,
    /// Set or clear the recipient. Pass `Some(Some(id))` to assign, `Some(None)` to clear.
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::double_option"
    )]
    pub recipient_id: Option<Option<Uuid>>,
}

pub fn create_blood_request(
    conn: &mut PgConnection,
    user: &User,
    payload: CreateBloodRequest,
) -> Result<BloodRequest, AppError> {
    // Role guard
    if user.role != Role::Hospital && user.role != Role::Hospital {
        return Err(AppError::Unauthorized(
            "Only hospitals can create blood requests".into(),
        ));
    }

    // Ownership verification
    use crate::schema::hospitals::dsl as hospitals_dsl;

    let owns_hospital = hospitals_dsl::hospitals
        .filter(hospitals_dsl::id.eq(payload.hospital_id))
        .filter(hospitals_dsl::user_id.eq(user.id))
        .select(hospitals_dsl::id)
        .first::<Uuid>(conn)
        .optional()?;
    if owns_hospital.is_none() {
        return Err(AppError::Unauthorized("Hospital not found".into()));
    }

    let new_request = diesel::insert_into(blood_requests)
        .values((
            hospital_id.eq(payload.hospital_id),
            donor_id.eq::<Option<Uuid>>(None),
            recipient_id.eq::<Option<Uuid>>(None),
            ref_id.eq(Uuid::new_v4().to_string()),
            units.eq(payload.units),
            blood_type.eq(payload.blood_type),
            urgency.eq(payload.urgency),
            timeline_status.eq(TimelineTypeEnum::RequestCreated),
            request_status.eq(RequestStatusTypeEnum::Confirmed),
            request_reason.eq(payload.request_reason),
            note.eq(payload.note),
            preferred_time.eq(payload.preferred_time),
        ))
        .returning(BloodRequest::as_select())
        .get_result(conn)?;

    Ok(new_request)
}

pub fn get_blood_requests(
    conn: &mut PgConnection,
    user: &User,
    filters: BloodRequestQuery,
) -> Result<Vec<BloodRequestResponse>, AppError> {
    use crate::schema::{blood_requests::dsl::*, hospitals::dsl as hospitals_dsl, users};

    // Create both aliases in a single call
    diesel::alias!(users as donor_user: DonorUser, users as patient_user: PatientUser);

    let mut query = blood_requests
        .left_join(donor_user.on(donor_user.field(users::id).nullable().eq(donor_id)))
        .left_join(patient_user.on(patient_user.field(users::id).nullable().eq(recipient_id)))
        .into_boxed();

    // Access control
    match user.role {
        Role::Hospital => {
            let hospital_id_owned = hospitals_dsl::hospitals
                .filter(hospitals_dsl::user_id.eq(user.id))
                .select(hospitals_dsl::id)
                .first::<Uuid>(conn)?;

            query = query.filter(hospital_id.eq(hospital_id_owned));
        }
        Role::Admin => {} // full access
        _ => return Err(AppError::Unauthorized("Access denied".into())),
    }

    // Filters
    if let Some(status) = filters.request_status {
        query = query.filter(request_status.eq(status));
    }

    if let Some(bt) = filters.blood_type {
        query = query.filter(blood_type.eq(bt));
    }

    if let Some(u) = filters.urgency {
        query = query.filter(urgency.eq(u));
    }

    let rows = query
        .order(created_at.desc())
        .load::<(BloodRequest, Option<User>, Option<User>)>(conn)?;

    Ok(rows
        .into_iter()
        .map(|(br, donor, patient)| BloodRequestResponse {
            blood_request: br,
            donor,
            patient,
        })
        .collect())
}

pub fn update_blood_request(
    conn: &mut PgConnection,
    request_id: Uuid,
    user: &User,
    payload: UpdateBloodRequest,
) -> Result<BloodRequest, AppError> {
    if user.role != Role::Hospital {
        return Err(AppError::Unauthorized(
            "Only hospitals can update blood requests".into(),
        ));
    }

    use crate::schema::hospitals::dsl as hospitals_dsl;

    let hospital_id_owned = hospitals_dsl::hospitals
        .filter(hospitals_dsl::user_id.eq(user.id))
        .select(hospitals_dsl::id)
        .first::<Uuid>(conn)
        .optional()?
        .ok_or_else(|| AppError::Unauthorized("Hospital not found".into()))?;

    let updated = diesel::update(
        blood_requests
            .filter(id.eq(request_id))
            .filter(hospital_id.eq(hospital_id_owned)),
    )
    .set(payload)
    .returning(BloodRequest::as_select())
    .get_result::<BloodRequest>(conn)
    .optional()?;

    updated.ok_or_else(|| AppError::NotFound("Blood request not found".into()))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DonorStats {
    /// Total number of completed blood donations
    pub total_donations: i64,
    /// Total volume donated in litres (assuming 450 ml per donation unit)
    pub total_litres_donated: f64,
    /// Donor's blood type from their patient profile, if recorded
    pub blood_type: Option<String>,
}

pub fn get_donor_stats(
    conn: &mut PgConnection,
    donor_user_id: Uuid,
) -> Result<DonorStats, AppError> {
    use crate::schema::blood_requests::dsl as br;
    use crate::schema::patients::dsl as p;

    // Count rows and sum units for requests where this user is the donor
    // and the donation was completed (donated_at is set).
    let (count, total_units): (i64, Option<i64>) = br::blood_requests
        .filter(br::donor_id.eq(donor_user_id))
        .filter(br::donated_at.is_not_null())
        .select((diesel::dsl::count(br::id), diesel::dsl::sum(br::units)))
        .get_result(conn)?;

    // 450 ml per standard donation unit → 0.45 L
    let total_litres_donated = total_units.unwrap_or(0) as f64 * 0.45;

    // Blood type lives on the patient profile
    let donor_blood_type: Option<String> = p::patients
        .filter(p::user_id.eq(donor_user_id))
        .select(p::blood_type)
        .first::<Option<String>>(conn)
        .optional()?
        .flatten();

    Ok(DonorStats {
        total_donations: count,
        total_litres_donated,
        blood_type: donor_blood_type,
    })
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DonationHistoryItem {
    pub ref_id: String,
    pub units: Option<i32>,
    pub request_status: Option<RequestStatusTypeEnum>,
    pub donated_at: Option<chrono::DateTime<Utc>>,
    pub cancelled_at: Option<chrono::DateTime<Utc>>,
}

pub fn get_donor_history(
    conn: &mut PgConnection,
    donor_user_id: Uuid,
) -> Result<Vec<DonationHistoryItem>, AppError> {
    use crate::schema::blood_requests::dsl as br;

    let rows = br::blood_requests
        .filter(br::donor_id.eq(donor_user_id))
        .select((
            br::ref_id,
            br::units,
            br::request_status,
            br::donated_at,
            br::cancelled_at,
        ))
        .order(br::created_at.desc())
        .load::<(
            String,
            Option<i32>,
            Option<RequestStatusTypeEnum>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        )>(conn)?;

    Ok(rows
        .into_iter()
        .map(
            |(h_ref_id, h_units, h_request_status, h_donated_at, h_cancelled_at)| {
                DonationHistoryItem {
                    ref_id: h_ref_id,
                    units: h_units,
                    request_status: h_request_status,
                    donated_at: h_donated_at,
                    cancelled_at: h_cancelled_at,
                }
            },
        )
        .collect())
}
