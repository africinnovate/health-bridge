import re

with open("src/handlers/appointments.rs", "r") as f:
    text = f.read()

# Replace RewardTypeEnum::Bonus with Applied / Earned
text = text.replace("RewardTypeEnum::Bonus, \n                    description: Some(\"Points applied for consultation discount\"", "RewardTypeEnum::Applied, \n                    description: Some(\"Points applied for consultation discount\"")
text = text.replace("RewardTypeEnum::Bonus, \n                            description: Some(\"Points refunded from cancelled appointment\"", "RewardTypeEnum::Earned, \n                            description: Some(\"Points refunded from cancelled appointment\"")

# Replace WalletTransactionTypeEnum::Earning with Deposit
text = text.replace("WalletTransactionTypeEnum::Earning", "WalletTransactionTypeEnum::Deposit")

# Fix JSON value type inference inside verify_appointment_payment
# From:
# if meta.get("paystack_reference") == Some(&serde_json::json!(payload.reference)) {
# To:
# if meta.get("paystack_reference").and_then(|v| v.as_str()) == Some(&payload.reference) {
text = text.replace(
    'if meta.get("paystack_reference") == Some(&serde_json::json!(payload.reference)) {',
    'if meta.as_object().and_then(|m| m.get("paystack_reference")).and_then(|v| v.as_str()) == Some(&payload.reference) {',
)

# And the other type inference errors:
# let total_payable = metadata.get("total_payable").and_then(|v| v.as_f64()).unwrap_or(0.0);
# To:
# let total_payable = metadata.as_object().and_then(|m| m.get("total_payable")).and_then(|v| v.as_f64()).unwrap_or(0.0);
text = text.replace(
    'let total_payable = metadata.get("total_payable").and_then(|v| v.as_f64()).unwrap_or(0.0);',
    'let total_payable = metadata.as_object().and_then(|m| m.get("total_payable")).and_then(|v| v.as_f64()).unwrap_or(0.0);'
)
text = text.replace(
    'let appt_id = metadata.get("appointment_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok())',
    'let appt_id = metadata.as_object().and_then(|m| m.get("appointment_id")).and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok())'
)

# Same inside cancel_appointment
text = text.replace(
    'let points_applied = metadata.get("points_applied").and_then(|v| v.as_i64()).unwrap_or(0);',
    'let points_applied = metadata.as_object().and_then(|m| m.get("points_applied")).and_then(|v| v.as_i64()).unwrap_or(0);'
)


with open("src/handlers/appointments.rs", "w") as f:
    f.write(text)

