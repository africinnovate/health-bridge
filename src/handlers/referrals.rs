use axum::{
    extract::{State, Extension},
    Json,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    models::{User, ReferralReward},
    schema::{users, referrals, referral_rewards, app_configs, appointments},
    utils::{
        enums::{ReferralStatusEnum, RewardTypeEnum, Role, AppointmentStatusEnum},
        response::ApiResponse,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub struct RewardHistoryResponse {
    pub id: Uuid,
    pub points: i32,
    pub reward_type: RewardTypeEnum,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferralStat {
    pub role: Role,
    pub pending: i64,
    pub active: i64,
    pub verified: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReferralSummaryResponse {
    pub total_points: i32,
    pub naira_equivalent: f64,
    pub reward_history: Vec<RewardHistoryResponse>,
    pub referral_stats: Vec<ReferralStat>,
    pub referral_code: String,
    pub referral_link: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateReferralLinkPayload {
    pub referral_link: String,
}

/// Get referral summary for the current user
#[utoipa::path(
    get,
    path = "/api/referrals",
    responses(
        (status = 200, body = ApiResponse<ReferralSummaryResponse>),
        (status = 401)
    ),
    tag = "referrals",
    security(("bearer_auth" = []))
)]
pub async fn get_referral_summary(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<ApiResponse<ReferralSummaryResponse>, AppError> {
    let mut conn = state.pool.get()?;

    // 1. Get total points
    let earned_points: i64 = referral_rewards::table
        .filter(referral_rewards::user_id.eq(user.id))
        .filter(referral_rewards::reward_type.eq(RewardTypeEnum::Earned))
        .select(diesel::dsl::sum(referral_rewards::points))
        .first::<Option<i64>>(&mut conn)?
        .unwrap_or(0);

    let applied_points: i64 = referral_rewards::table
        .filter(referral_rewards::user_id.eq(user.id))
        .filter(referral_rewards::reward_type.eq(RewardTypeEnum::Applied))
        .select(diesel::dsl::sum(referral_rewards::points))
        .first::<Option<i64>>(&mut conn)?
        .unwrap_or(0);

    let total_points = (earned_points - applied_points) as i32;

    // 2. Get Naira equivalent from config
    let point_value_str = app_configs::table
        .filter(app_configs::key.eq("reward_point"))
        .select(app_configs::value)
        .first::<String>(&mut conn)
        .unwrap_or_else(|_| "0".to_string());
    
    let point_value: f64 = point_value_str.parse().unwrap_or(0.0);
    let naira_equivalent = total_points as f64 * point_value;

    // 3. Get Reward History
    let history = referral_rewards::table
        .filter(referral_rewards::user_id.eq(user.id))
        .order(referral_rewards::created_at.desc())
        .limit(10)
        .load::<ReferralReward>(&mut conn)?
        .into_iter()
        .map(|r| RewardHistoryResponse {
            id: r.id,
            points: r.points,
            reward_type: r.reward_type,
            description: r.description,
            created_at: r.created_at,
        })
        .collect();

    // 4. Get Referral Stats
    // This is a bit complex. We need to categorize by role and check status.
    // Active = 1+ confirmed appointment.
    // Verified = specialist.verified = true.
    
    let referred_users = referrals::table
        .inner_join(users::table.on(referrals::referred_user_id.eq(users::id)))
        .filter(referrals::referrer_id.eq(user.id))
        .select((users::all_columns, referrals::status))
        .load::<(User, ReferralStatusEnum)>(&mut conn)?;

    let mut stats_map: std::collections::HashMap<Role, (i64, i64, i64)> = std::collections::HashMap::new();

    for (referred_user, _status) in referred_users {
        let (pending, active, verified) = stats_map.entry(referred_user.role).or_insert((0, 0, 0));
        
        // Check if active (1+ confirmed appointment)
        let is_active = appointments::table
            .filter(appointments::user_id.eq(referred_user.id))
            .filter(appointments::status.eq(AppointmentStatusEnum::Confirmed))
            .count()
            .get_result::<i64>(&mut conn)? > 0;

        if is_active {
            *active += 1;
        } else {
            *pending += 1;
        }

        // Check if verified (Specialist only)
        if referred_user.role == Role::Specialist {
            let is_verified = crate::schema::specialists::table
                .filter(crate::schema::specialists::user_id.eq(referred_user.id))
                .select(crate::schema::specialists::verified)
                .first::<bool>(&mut conn)
                .unwrap_or(false);
            if is_verified {
                *verified += 1;
            }
        }
    }

    let referral_stats = stats_map
        .into_iter()
        .map(|(role, (pending, active, verified))| ReferralStat {
            role,
            pending,
            active,
            verified,
        })
        .collect();

    Ok(ApiResponse::success(ReferralSummaryResponse {
        total_points,
        naira_equivalent,
        reward_history: history,
        referral_stats,
        referral_code: user.referral_code,
        referral_link: user.referral_link,
    }))
}

/// Update user's referral link
#[utoipa::path(
    put,
    path = "/api/referrals/link",
    request_body = UpdateReferralLinkPayload,
    responses(
        (status = 200, body = ApiResponse<String>),
        (status = 401)
    ),
    tag = "referrals",
    security(("bearer_auth" = []))
)]
pub async fn update_referral_link(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UpdateReferralLinkPayload>,
) -> Result<ApiResponse<String>, AppError> {
    let mut conn = state.pool.get()?;

    diesel::update(users::table.find(user.id))
        .set(users::referral_link.eq(payload.referral_link))
        .execute(&mut conn)?;

    Ok(ApiResponse::success("Referral link updated successfully".into()))
}
