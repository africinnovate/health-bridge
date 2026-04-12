import re

with open("src/handlers/appointments.rs", "r") as f:
    text = f.read()

# Replace complete_appointment
old_complete = r'pub async fn complete_appointment\(.+?\}\n'
new_complete = """pub async fn complete_appointment(
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
"""
text = re.sub(old_complete, new_complete, text, flags=re.DOTALL)

# Replace cancel_appointment
old_cancel = r'pub async fn cancel_appointment\(.+?\}\n'
new_cancel = """pub async fn cancel_appointment(
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
                let total_payable = metadata.get("total_payable").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let points_applied = metadata.get("points_applied").and_then(|v| v.as_i64()).unwrap_or(0);

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
                            reward_type: RewardTypeEnum::Bonus, 
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
"""
text = re.sub(old_cancel, new_cancel, text, flags=re.DOTALL)

with open("src/handlers/appointments.rs", "w") as f:
    f.write(text)

print("Patched cancel and complete functions.")
