use diesel::prelude::*;
use diesel::AsChangeset;
use diesel::associations::HasTable;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use tracing::{info, error};

use crate::utils::enums::Role;
use crate::{
    models::{BloodRequest, User},
    schema::blood_requests::dsl::*,
    error::AppError,
    utils::enums::{UrgencyTypeEnum, BloodTypeEnum, TimelineTypeEnum, RequestStatusTypeEnum},
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
    use crate::schema::{
        blood_requests::dsl::*,
        hospitals::dsl as hospitals_dsl,
        users,
    };

    // Create both aliases in a single call
    diesel::alias!(users as donor_user: DonorUser, users as patient_user: PatientUser);

    let mut query = blood_requests
        .left_join(
            donor_user.on(donor_user.field(users::id).nullable().eq(donor_id))
        )
        .left_join(
            patient_user.on(patient_user.field(users::id).nullable().eq(recipient_id))
        )
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


