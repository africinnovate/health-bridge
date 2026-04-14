use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    hospitals::{
        self,
        appointments::{AppointmentQuery, AppointmentResponse},
    },
    models::{
        Appointment, ConsultationPackage, NewReferralReward, User,
        WalletTransaction, NewWalletTransaction, ConsultationType,
    },
    schema::{
        app_configs, appointments as appointments_schema, consultation_packages, consultation_types,
        referral_rewards, wallets, wallet_transactions,
    },
    utils::{
        enums::{
            AppointmentStatusEnum, AppointmentTypeEnum, RewardTypeEnum, WalletTransactionStatusEnum,
            WalletTransactionTypeEnum,
        },
        response::ApiResponse,
    },
};

// Helper for configs
fn get_config_value(conn: &mut PgConnection, config_key: &str) -> Result<String, AppError> {
    app_configs::table
        .filter(app_configs::key.eq(config_key))
        .select(app_configs::value)
        .first::<String>(conn)
        .map_err(|_| AppError::InternalServerError)
}

async fn perform_platform_transfer(
    state: &AppState,
    amount: BigDecimal,
    pool_conn: &mut PgConnection,
) -> Result<(), AppError> {
    let account_number = get_config_value(pool_conn, "app_account_number")?;
    let account_name = get_config_value(pool_conn, "app_account_name")?;
    let bank_code = get_config_value(pool_conn, "app_account_code")?;

    use crate::services::paystack::CreateTransferRecipientRequest;
    
    let recipient_req = CreateTransferRecipientRequest {
        r#type: "nuban".into(),
        name: account_name,
        account_number,
        bank_code,
        currency: "NGN".into(),
    };
    
    let recipient_res = state.paystack_service.create_transfer_recipient(recipient_req).await?;

    let amount_kobo = (amount * BigDecimal::from(100))
        .to_u64()
        .ok_or_else(|| AppError::BadRequest("Invalid amount".into()))?;

    use crate::services::paystack::InitiateTransferRequest;
    let transfer_req = InitiateTransferRequest {
        source: "balance".into(),
        amount: amount_kobo,
        recipient: recipient_res.recipient_code,
        reason: Some("Platform Consultation Transfer".into()),
        reference: Some(format!("RUBI-{}-{}", Uuid::new_v4(), chrono::Utc::now().timestamp())),
    };

    let _ = state.paystack_service.initiate_transfer(transfer_req).await;
    Ok(())
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAppointmentPayload {
    pub package_id: Option<Uuid>,
    pub use_wallet: Option<bool>,
    pub apply_points: Option<i32>,
    pub blood_request_id: Option<Uuid>,
    pub specialist_id: Uuid,
    pub appointment_type: AppointmentTypeEnum,
    pub scheduled_time: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyPointsPayload {
    pub package_id: Uuid,
    pub apply_points: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplyPointsResponse {
    #[schema(value_type = String)]
    pub original_fee: BigDecimal,
    #[schema(value_type = String)]
    pub discount: BigDecimal,
    #[schema(value_type = String)]
    pub total_payable: BigDecimal,
    pub available_points: i32,
}

#[derive(Serialize, ToSchema)]
pub struct CreateAppointmentResponse {
    pub appointment: Appointment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyAppointmentPaymentPayload {
    pub reference: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RescheduleAppointmentPayload {
    pub scheduled_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelAppointmentPayload {
    pub reason: Option<String>,
}

/// Create appointment (donor or patient initiates)
#[utoipa::path(
    post,
    path = "/api/appointments/create",
    request_body = CreateAppointmentPayload,
    responses(
        (status = 201, body = ApiResponse<CreateAppointmentResponse>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn create_appointment(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<CreateAppointmentPayload>,
) -> Result<ApiResponse<CreateAppointmentResponse>, AppError> {
    let mut conn = state.pool.get()?;
    
    let mut fee = BigDecimal::from(0);
    if let Some(pkg_id) = payload.package_id {
        if let Ok((pkg, ctype)) = consultation_packages::table
            .find(pkg_id)
            .inner_join(consultation_types::table)
            .first::<(ConsultationPackage, ConsultationType)>(&mut conn) {
                fee = pkg.custom_price.unwrap_or(ctype.base_price);
            }
    }

    let reward_point_val: i32 = get_config_value(&mut conn, "reward_point")
        .and_then(|v| v.parse().map_err(|_| AppError::InternalServerError))
        .unwrap_or(0);
        
    let platform_percent: BigDecimal = get_config_value(&mut conn, "consultation_fee_percentage")
        .and_then(|v| v.parse().map_err(|_| AppError::InternalServerError))
        .unwrap_or_else(|_| BigDecimal::from(20));

    let apply_points = payload.apply_points.unwrap_or(0);
    let discount = BigDecimal::from(apply_points * reward_point_val);
    
    let total_payable = if fee > discount { fee.clone() - discount.clone() } else { BigDecimal::from(0) };
    let use_wallet = payload.use_wallet.unwrap_or(false);

    let pay_reference = format!("PAY-{}", Uuid::new_v4());
    let mut authorization_url = None;

    if !use_wallet && total_payable > BigDecimal::from(0) {
        let amount_kobo = (total_payable.clone() * BigDecimal::from(100))
            .to_u64()
            .ok_or_else(|| AppError::BadRequest("Invalid amount".into()))?;

        let ps_req = crate::services::paystack::InitializeTransactionRequest {
            email: user.email.clone(),
            amount: amount_kobo,
            reference: Some(pay_reference.clone()),
            callback_url: None, 
            metadata: Some(serde_json::json!({
                "type": "appointment",
                "specialist_id": payload.specialist_id,
            })),
        };

        let ps_res = state.paystack_service.initialize_transaction(ps_req).await?;
        authorization_url = Some(ps_res.authorization_url);
    }

    // Database transaction
    let appointment = conn.transaction::<_, AppError, _>(|c| {
        if use_wallet && total_payable > BigDecimal::from(0) {
            let user_wallet = crate::handlers::wallets::get_or_create_wallet(c, user.id)?;
            if user_wallet.balance < total_payable {
                return Err(AppError::BadRequest("Insufficient wallet balance".into()));
            }
            
            diesel::update(wallets::table.find(user_wallet.id))
                .set(wallets::balance.eq(wallets::balance - &total_payable))
                .execute(c)?;
                
            diesel::insert_into(wallet_transactions::table)
                .values(&NewWalletTransaction {
                    wallet_id: user_wallet.id,
                    amount: total_payable.clone(),
                    transaction_type: WalletTransactionTypeEnum::Transfer,
                    status: WalletTransactionStatusEnum::Successful,
                    reference: format!("APPT-PAY-{}", Uuid::new_v4()),
                    provider: "wallet".into(),
                    description: Some("Consultation fee payment".into()),
                    metadata: None,
                })
                .execute(c)?;
        }

        if apply_points > 0 {
            diesel::insert_into(referral_rewards::table)
                .values(&NewReferralReward {
                    user_id: user.id,
                    referral_id: None,
                    points: -apply_points,
                    reward_type: RewardTypeEnum::Applied, 
                    description: Some("Points applied for consultation discount".into()),
                })
                .execute(c)?;
        }

        let create_payload = hospitals::appointments::CreateAppointment {
            blood_request_id: payload.blood_request_id,
            specialist_id: payload.specialist_id,
            appointment_type: payload.appointment_type,
            scheduled_time: payload.scheduled_time,
            notes: payload.notes.clone(),
        };
        let appt = hospitals::appointments::create_appointment(c, &user, create_payload)?;
        
        let specialist_wallet = crate::handlers::wallets::get_or_create_wallet(c, payload.specialist_id)?;
        
        let platform_cut = (fee.clone() * platform_percent.clone()) / BigDecimal::from(100);
        let specialist_cut = fee.clone() - platform_cut.clone();

        diesel::insert_into(wallet_transactions::table)
            .values(&NewWalletTransaction {
                wallet_id: specialist_wallet.id,
                amount: specialist_cut.clone(),
                transaction_type: WalletTransactionTypeEnum::Deposit,
                status: WalletTransactionStatusEnum::Pending, 
                reference: format!("ESCROW-{}", appt.id),
                provider: "system".into(),
                description: Some("Pending consultation earning".into()),
                metadata: Some(serde_json::json!({
                    "appointment_id": appt.id,
                    "platform_cut": platform_cut.to_f64().unwrap_or(0.0),
                    "total_payable": total_payable.to_f64().unwrap_or(0.0),
                    "points_applied": apply_points,
                    "paystack_reference": pay_reference,
                    "paid_via_wallet": use_wallet
                })),
            })
            .execute(c)?;

        Ok(appt)
    })?;

    if use_wallet && total_payable > BigDecimal::from(0) {
        let _ = perform_platform_transfer(&state, total_payable.clone(), &mut conn).await;
    }

    Ok(ApiResponse::created(
        "Appointment created successfully",
        CreateAppointmentResponse {
            appointment,
            authorization_url,
            reference: (!use_wallet).then_some(pay_reference),
        },
    ))
}

#[utoipa::path(
    get,
    path = "/api/appointments",
    params(
        ("appointment_type" = Option<AppointmentTypeEnum>, Query),
        ("status" = Option<AppointmentStatusEnum>, Query),
        ("timeline" = Option<String>, Query, description = "Filter by timeframe (today, this_week, this_month, upcoming)"),
        ("specialist_id" = Option<Uuid>, Query)
    ),
    responses(
        (status = 200, body = ApiResponse<Vec<AppointmentResponse>>),
        (status = 401)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn get_appointments(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(filters): Query<AppointmentQuery>,
) -> Result<ApiResponse<Vec<AppointmentResponse>>, AppError> {
    let mut conn = state.pool.get()?;
    let results = hospitals::appointments::get_appointments(&mut conn, &user, filters)?;
    Ok(ApiResponse::success(results))
}

/// Confirm appointment
///
/// Used by Hospital staff, Specialists, or Admins to confirm an appointment
#[utoipa::path(
    put,
    path = "/api/appointments/confirm/{appointment_id}",
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn confirm_appointment(
    Path(id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appt = hospitals::appointments::confirm_appointment(&mut conn, id, &user)?;
    Ok(ApiResponse::success(appt))
}

/// Reschedule an appointment
#[utoipa::path(
    put,
    path = "/api/appointments/reschedule/{appointment_id}",
    request_body = RescheduleAppointmentPayload,
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn reschedule_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(payload): Json<RescheduleAppointmentPayload>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    let appointment = hospitals::appointments::reschedule_appointment(
        &mut conn,
        appointment_id,
        &user,
        payload.scheduled_time,
    )?;
    Ok(ApiResponse::success(appointment))
}

/// Cancel Appointment
#[utoipa::path(
    put,
    path = "/api/appointments/cancel/{appointment_id}",
    request_body = CancelAppointmentPayload,
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn cancel_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(payload): Json<CancelAppointmentPayload>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    
    let appointment = conn.transaction::<_, AppError, _>(|c| {
        let existing_appt = appointments_schema::table
            .find(appointment_id)
            .first::<Appointment>(c)?;
            
        let window_hours: i64 = get_config_value(c, "appointment_cancel_window")
            .and_then(|v| v.parse().map_err(|_| AppError::InternalServerError))
            .unwrap_or(24);

        if chrono::Utc::now() > existing_appt.scheduled_time - chrono::Duration::hours(window_hours) {
            return Err(AppError::BadRequest("Appointment cancellation window has passed".into()));
        }

        let appt = hospitals::appointments::cancel_appointment(c, appointment_id, &user, payload.reason)?;

        let escrow_tx = wallet_transactions::table
            .filter(wallet_transactions::reference.eq(format!("ESCROW-{}", appt.id)))
            .filter(wallet_transactions::status.eq(WalletTransactionStatusEnum::Pending))
            .first::<WalletTransaction>(c)
            .optional()?;

        if let Some(target_tx) = escrow_tx {
            diesel::update(wallet_transactions::table.find(target_tx.id))
                .set(wallet_transactions::status.eq(WalletTransactionStatusEnum::Failed))
                .execute(c)?;

            if let Some(metadata) = target_tx.metadata {
                let total_payable = metadata.as_object().and_then(|m| m.get("total_payable")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let points_applied = metadata.as_object().and_then(|m| m.get("points_applied")).and_then(|v| v.as_i64()).unwrap_or(0);

                let patient_wallet = crate::handlers::wallets::get_or_create_wallet(c, appt.user_id)?;

                if total_payable > 0.0 {
                    let refund_amount = BigDecimal::from_f64(total_payable).unwrap_or(BigDecimal::from(0));
                    
                    diesel::update(wallets::table.find(patient_wallet.id))
                        .set(wallets::balance.eq(wallets::balance + &refund_amount))
                        .execute(c)?;

                    diesel::insert_into(wallet_transactions::table)
                        .values(&NewWalletTransaction {
                            wallet_id: patient_wallet.id,
                            amount: refund_amount,
                            transaction_type: WalletTransactionTypeEnum::Refund,
                            status: WalletTransactionStatusEnum::Successful,
                            reference: format!("REFUND-{}", appt.id),
                            provider: "system".into(),
                            description: Some("Consultation fee refund".into()),
                            metadata: None,
                        })
                        .execute(c)?;
                }

                if points_applied > 0 {
                    diesel::insert_into(referral_rewards::table)
                        .values(&NewReferralReward {
                            user_id: appt.user_id,
                            referral_id: None,
                            points: points_applied as i32,
                            reward_type: RewardTypeEnum::Earned, 
                            description: Some("Points refunded from cancelled appointment".into()),
                        })
                        .execute(c)?;
                }
            }
        }

        Ok(appt)
    })?;

    Ok(ApiResponse::success(appointment))
}

/// Complete appointment. Used only by Hospital
#[utoipa::path(
    put,
    path = "/api/appointments/complete/{appointment_id}",
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 401),
        (status = 404)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn complete_appointment(
    Path(appointment_id): Path<Uuid>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;
    
    let appointment = conn.transaction::<_, AppError, _>(|c| {
        let appt = hospitals::appointments::complete_appointment(c, appointment_id, &user)?;

        let escrow_tx = wallet_transactions::table
            .filter(wallet_transactions::reference.eq(format!("ESCROW-{}", appt.id)))
            .filter(wallet_transactions::status.eq(WalletTransactionStatusEnum::Pending))
            .first::<WalletTransaction>(c)
            .optional()?;

        if let Some(target_tx) = escrow_tx {
            diesel::update(wallet_transactions::table.find(target_tx.id))
                .set(wallet_transactions::status.eq(WalletTransactionStatusEnum::Successful))
                .execute(c)?;
                
            diesel::update(wallets::table.find(target_tx.wallet_id))
                .set(wallets::balance.eq(wallets::balance + &target_tx.amount))
                .execute(c)?;
        }

        Ok(appt)
    })?;

    Ok(ApiResponse::success(appointment))
}

#[utoipa::path(
    post,
    path = "/api/appointments/verify-payment",
    request_body = VerifyAppointmentPaymentPayload,
    responses(
        (status = 200, body = ApiResponse<Appointment>),
        (status = 400),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn verify_appointment_payment(
    State(state): State<AppState>,
    Json(payload): Json<VerifyAppointmentPaymentPayload>,
) -> Result<ApiResponse<Appointment>, AppError> {
    let mut conn = state.pool.get()?;

    let ps_res = state.paystack_service.verify_transaction(&payload.reference).await?;
    
    if ps_res.status != "success" {
        return Err(AppError::BadRequest("Payment not successful".into()));
    }

    let pending_transactions = wallet_transactions::table
        .filter(wallet_transactions::status.eq(WalletTransactionStatusEnum::Pending))
        .filter(wallet_transactions::transaction_type.eq(WalletTransactionTypeEnum::Deposit))
        .load::<crate::models::WalletTransaction>(&mut conn)?;
        
    let mut matching_tx = None;
    for tx in pending_transactions {
        if let Some(meta) = &tx.metadata {
            if meta.as_object().and_then(|m| m.get("paystack_reference")).and_then(|v| v.as_str()) == Some(&payload.reference) {
                matching_tx = Some(tx);
                break;
            }
        }
    }

    let target_tx = matching_tx.ok_or_else(|| AppError::NotFound("Pending appointment payment not found".into()))?;
    
    let metadata = target_tx.metadata.unwrap();
    let total_payable = metadata.as_object().and_then(|m| m.get("total_payable")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let appt_id = metadata.as_object().and_then(|m| m.get("appointment_id")).and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok())
        .ok_or_else(|| AppError::InternalServerError)?;

    if total_payable > 0.0 {
        use bigdecimal::FromPrimitive;
        let _ = perform_platform_transfer(&state, BigDecimal::from_f64(total_payable).unwrap_or(BigDecimal::from(0)), &mut conn).await;
    }

    let appointment = appointments_schema::table
        .find(appt_id)
        .first::<Appointment>(&mut conn)?;

    Ok(ApiResponse::success(appointment))
}

/// Preview fee after applying points
#[utoipa::path(
    post,
    path = "/api/appointments/apply-points",
    request_body = ApplyPointsPayload,
    responses(
        (status = 200, body = ApiResponse<ApplyPointsResponse>),
        (status = 401),
        (status = 404),
        (status = 500)
    ),
    tag = "appointments",
    security(("bearer_auth" = []))
)]
pub async fn apply_points_preview(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(payload): Json<ApplyPointsPayload>,
) -> Result<ApiResponse<ApplyPointsResponse>, AppError> {
    let mut conn = state.pool.get()?;
    
    // 1. Get package fee
    let (pkg, ctype) = consultation_packages::table
        .find(payload.package_id)
        .inner_join(consultation_types::table)
        .first::<(ConsultationPackage, ConsultationType)>(&mut conn)
        .map_err(|_| AppError::NotFound("Package not found".into()))?;
    
    let fee = pkg.custom_price.unwrap_or(ctype.base_price);

    // 2. Get point value
    let reward_point_val: i32 = get_config_value(&mut conn, "reward_point")
        .and_then(|v| v.parse().map_err(|_| AppError::InternalServerError))
        .unwrap_or(0);

    // 3. Get user's available points
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

    let total_available_points = (earned_points + applied_points) as i32;

    if payload.apply_points > total_available_points {
        return Err(AppError::BadRequest(format!("Insufficient points. Available: {}", total_available_points)));
    }

    let discount = BigDecimal::from(payload.apply_points * reward_point_val);
    let total_payable = if fee > discount { fee.clone() - discount.clone() } else { BigDecimal::from(0) };

    Ok(ApiResponse::success(ApplyPointsResponse {
        original_fee: fee,
        discount,
        total_payable,
        available_points: total_available_points,
    }))
}

