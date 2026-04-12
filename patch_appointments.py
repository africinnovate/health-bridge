import re

with open("src/handlers/appointments.rs", "r") as f:
    text = f.read()

# 1. Imports
imports_replacement = """use axum::{
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
    hospitals,
    models::{
        Appointment, AppConfig, ConsultationPackage, ReferralReward, NewReferralReward, User,
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
"""

text = re.sub(r'use axum.*?::ApiResponse,\n    \},\n\};\n', imports_replacement, text, flags=re.DOTALL)

# Remove the old CreateAppointment struct path macro definition since we changed the body signature
old_path_macro = r'#\[utoipa::path\([^\]]+create_appointment.+?pub async fn create_appointment.*?\n\}'

new_create_appointment = """#[utoipa::path(
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
                    reward_type: RewardTypeEnum::Bonus, 
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
        let specialist_cut = fee.clone() - platform_cut;

        diesel::insert_into(wallet_transactions::table)
            .values(&NewWalletTransaction {
                wallet_id: specialist_wallet.id,
                amount: specialist_cut.clone(),
                transaction_type: WalletTransactionTypeEnum::Earning,
                status: WalletTransactionStatusEnum::Pending, 
                reference: format!("ESCROW-{}", appt.id),
                provider: "system".into(),
                description: Some("Pending consultation earning".into()),
                metadata: Some(serde_json::json!({
                    "appointment_id": appt.id,
                    "platform_cut": platform_cut.to_f64().unwrap_or(0.0),
                    "total_payable": total_payable.to_f64().unwrap_or(0.0),
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
        .filter(wallet_transactions::transaction_type.eq(WalletTransactionTypeEnum::Earning))
        .load::<crate::models::WalletTransaction>(&mut conn)?;
        
    let mut matching_tx = None;
    for tx in pending_transactions {
        if let Some(meta) = &tx.metadata {
            if meta.get("paystack_reference") == Some(&serde_json::json!(payload.reference)) {
                matching_tx = Some(tx);
                break;
            }
        }
    }

    let target_tx = matching_tx.ok_or_else(|| AppError::NotFound("Pending appointment payment not found".into()))?;
    
    let metadata = target_tx.metadata.unwrap();
    let total_payable = metadata.get("total_payable").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let appt_id = metadata.get("appointment_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok())
        .ok_or_else(|| AppError::InternalServerError)?;

    if total_payable > 0.0 {
        let _ = perform_platform_transfer(&state, BigDecimal::from_f64(total_payable).unwrap_or(BigDecimal::from(0)), &mut conn).await;
    }

    let appointment = appointments_schema::table
        .find(appt_id)
        .first::<Appointment>(&mut conn)?;

    // We keep the escort transaction Pending until completion. It's just now confirmed to be funded.
    // If you want to change its status to mark it as funded, you can add another field. For now it remains Pending.

    Ok(ApiResponse::success(appointment))
}
"""

text = re.sub(old_path_macro, new_create_appointment, text, flags=re.DOTALL)

with open("src/handlers/appointments.rs", "w") as f:
    f.write(text)

print("Patched appointments.rs")
