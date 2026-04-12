use axum::{Extension, Json, extract::{Path, Query, State}};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    AppState,
    error::AppError,
    models::{
        User, Wallet, WalletTransaction, NewWalletTransaction,
        BankAccount, NewBankAccount,
    },
    schema::{wallets, wallet_transactions, bank_accounts},
    services::paystack::{
        InitializeTransactionRequest, CreateTransferRecipientRequest, InitiateTransferRequest,
    },
    utils::{
        enums::{WalletTransactionStatusEnum, WalletTransactionTypeEnum},
        response::ApiResponse,
    },
};

#[derive(Serialize, ToSchema)]
pub struct WalletSummary {
    #[schema(value_type = String)]
    pub balance: BigDecimal,
    pub currency: String,
    pub recent_transactions: Vec<WalletTransaction>,
}

#[derive(Deserialize, ToSchema)]
pub struct DepositRequest {
    #[schema(value_type = String)]
    pub amount: BigDecimal,
}

#[derive(Serialize, ToSchema)]
pub struct DepositInitializeResponse {
    pub authorization_url: String,
    pub reference: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AddBankAccountRequest {
    pub account_number: String,
    pub bank_code: String,
    pub bank_name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct WithdrawalRequest {
    #[schema(value_type = String)]
    pub amount: BigDecimal,
    pub bank_account_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct TransactionFilters {
    /// Optional text search against reference, description, or provider
    pub search: Option<String>,

    /// Filter by transaction status (pending, successful, failed, reversed)
    pub status: Option<WalletTransactionStatusEnum>,

    /// Filter by transaction type (deposit, withdrawal, transfer, refund, reward)
    pub transaction_type: Option<WalletTransactionTypeEnum>,

    /// Filter by start date (inclusive, YYYY-MM-DD)
    pub date_from: Option<NaiveDate>,

    /// Filter by end date (inclusive, YYYY-MM-DD)
    pub date_to: Option<NaiveDate>,

    /// Minimum amount filter, e.g. "500.00"
    pub amount_min: Option<String>,

    /// Maximum amount filter, e.g. "10000.00"
    pub amount_max: Option<String>,

    /// Page number (starts from 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Items per page (default: 20, max: 100)
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

#[derive(Serialize, ToSchema)]
pub struct TransactionListResponse {
    pub data: Vec<WalletTransaction>,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Get wallet balance and transaction history
#[utoipa::path(
    get,
    path = "/api/wallets",
    responses(
        (status = 200, body = ApiResponse<WalletSummary>),
        (status = 401),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn get_wallet_summary(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
) -> Result<ApiResponse<WalletSummary>, AppError> {
    let mut conn = state.pool.get()?;

    let wallet = get_or_create_wallet(&mut conn, current_user.id)?;

    let transactions = wallet_transactions::table
        .filter(wallet_transactions::wallet_id.eq(wallet.id))
        .order(wallet_transactions::created_at.desc())
        .limit(10)
        .load::<WalletTransaction>(&mut conn)?;

    Ok(ApiResponse::success(WalletSummary {
        balance: wallet.balance,
        currency: wallet.currency,
        recent_transactions: transactions,
    }))
}

/// Initialize a deposit via Paystack
#[utoipa::path(
    post,
    path = "/api/wallets/deposit/initialize",
    request_body = DepositRequest,
    responses(
        (status = 200, body = ApiResponse<DepositInitializeResponse>),
        (status = 400),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn deposit_initialize(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<DepositRequest>,
) -> Result<ApiResponse<DepositInitializeResponse>, AppError> {
    if payload.amount <= BigDecimal::from(0) {
        return Err(AppError::BadRequest("Amount must be greater than zero".into()));
    }

    let mut conn = state.pool.get()?;
    let wallet = get_or_create_wallet(&mut conn, current_user.id)?;

    // Paystack amount is in kobo (NGN 1.00 = 100 kobo)
    let amount_kobo = (payload.amount.clone() * BigDecimal::from(100))
        .to_u64()
        .ok_or_else(|| AppError::BadRequest("Invalid amount".into()))?;

    let reference = format!("DEP-{}", Uuid::new_v4());

    let ps_req = InitializeTransactionRequest {
        email: current_user.email,
        amount: amount_kobo,
        reference: Some(reference.clone()),
        callback_url: None, // Frontend will handle redirection or polling
        metadata: Some(serde_json::json!({
            "wallet_id": wallet.id,
            "type": "deposit"
        })),
    };

    let ps_res = state.paystack_service.initialize_transaction(ps_req).await?;

    // Create a pending transaction record
    diesel::insert_into(wallet_transactions::table)
        .values(&NewWalletTransaction {
            wallet_id: wallet.id,
            amount: payload.amount,
            transaction_type: WalletTransactionTypeEnum::Deposit,
            status: WalletTransactionStatusEnum::Pending,
            reference: reference.clone(),
            provider: "paystack".into(),
            description: Some("Wallet funding via Paystack".into()),
            metadata: None,
        })
        .execute(&mut conn)?;

    Ok(ApiResponse::success(DepositInitializeResponse {
        authorization_url: ps_res.authorization_url,
        reference: reference,
    }))
}

/// Verify a deposit status
#[utoipa::path(
    get,
    path = "/api/wallets/deposit/verify/{reference}",
    responses(
        (status = 200, body = ApiResponse<WalletTransaction>),
        (status = 400),
        (status = 404),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn deposit_verify(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Path(reference): Path<String>,
) -> Result<ApiResponse<WalletTransaction>, AppError> {
    let mut conn = state.pool.get()?;

    let transaction = wallet_transactions::table
        .filter(wallet_transactions::reference.eq(&reference))
        .first::<WalletTransaction>(&mut conn)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

    // Ensure the transaction belongs to the current user's wallet
    let wallet = wallets::table
        .filter(wallets::id.eq(transaction.wallet_id))
        .filter(wallets::user_id.eq(current_user.id))
        .first::<Wallet>(&mut conn)
        .optional()?
        .ok_or_else(|| AppError::Unauthorized("Access denied".into()))?;

    if transaction.status != WalletTransactionStatusEnum::Pending {
        return Ok(ApiResponse::success(transaction));
    }

    let ps_res = state.paystack_service.verify_transaction(&reference).await?;

    if ps_res.status == "success" {
        conn.transaction::<_, AppError, _>(|c| {
            // Update transaction status
            let updated_tx = diesel::update(wallet_transactions::table.filter(wallet_transactions::id.eq(transaction.id)))
                .set(wallet_transactions::status.eq(WalletTransactionStatusEnum::Successful))
                .get_result::<WalletTransaction>(c)?;

            // Update wallet balance
            diesel::update(wallets::table.filter(wallets::id.eq(wallet.id)))
                .set(wallets::balance.eq(wallets::balance + &transaction.amount))
                .execute(c)?;

            Ok(ApiResponse::success(updated_tx))
        })
    } else if ps_res.status == "failed" {
        let updated_tx = diesel::update(wallet_transactions::table.filter(wallet_transactions::id.eq(transaction.id)))
            .set(wallet_transactions::status.eq(WalletTransactionStatusEnum::Failed))
            .get_result::<WalletTransaction>(&mut conn)?;
        Ok(ApiResponse::success(updated_tx))
    } else {
        Ok(ApiResponse::success(transaction))
    }
}

/// Get list of supported banks
#[utoipa::path(
    get,
    path = "/api/wallets/banks",
    responses(
        (status = 200, body = ApiResponse<Vec<crate::services::paystack::Bank>>),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn get_banks(
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<crate::services::paystack::Bank>>, AppError> {
    let banks = state.paystack_service.list_banks().await?;
    Ok(ApiResponse::success(banks))
}

/// Add bank account for withdrawals
#[utoipa::path(
    post,
    path = "/api/wallets/bank-accounts",
    request_body = AddBankAccountRequest,
    responses(
        (status = 200, body = ApiResponse<BankAccount>),
        (status = 400),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn add_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<AddBankAccountRequest>,
) -> Result<ApiResponse<BankAccount>, AppError> {
    let mut conn = state.pool.get()?;

    // Resolve account name via Paystack
    let resolve_res = state.paystack_service.resolve_account(&payload.account_number, &payload.bank_code).await?;

    // Create transfer recipient on Paystack
    let recipient_req = CreateTransferRecipientRequest {
        r#type: "nuban".into(),
        name: resolve_res.account_name.clone(),
        account_number: payload.account_number.clone(),
        bank_code: payload.bank_code.clone(),
        currency: "NGN".into(),
    };

    let recipient_res = state.paystack_service.create_transfer_recipient(recipient_req).await?;

    let new_bank_account = diesel::insert_into(bank_accounts::table)
        .values(&NewBankAccount {
            user_id: current_user.id,
            account_number: &payload.account_number,
            bank_code: &payload.bank_code,
            bank_name: &payload.bank_name,
            account_name: &resolve_res.account_name,
            recipient_code: &recipient_res.recipient_code,
            is_default: false,
        })
        .get_result::<BankAccount>(&mut conn)?;

    Ok(ApiResponse::success_with_message("Bank account added successfully", new_bank_account))
}

/// Initiate a withdrawal
#[utoipa::path(
    post,
    path = "/api/wallets/withdraw",
    request_body = WithdrawalRequest,
    responses(
        (status = 200, body = ApiResponse<WalletTransaction>),
        (status = 400),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn withdraw_funds(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<WithdrawalRequest>,
) -> Result<ApiResponse<WalletTransaction>, AppError> {
    if payload.amount <= BigDecimal::from(0) {
        return Err(AppError::BadRequest("Amount must be greater than zero".into()));
    }

    let mut conn = state.pool.get()?;
    let wallet = get_or_create_wallet(&mut conn, current_user.id)?;

    if wallet.balance < payload.amount {
        return Err(AppError::BadRequest("Insufficient funds".into()));
    }

    let bank_account = bank_accounts::table
        .filter(bank_accounts::id.eq(payload.bank_account_id))
        .filter(bank_accounts::user_id.eq(current_user.id))
        .first::<BankAccount>(&mut conn)
        .optional()?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    let amount_kobo = (payload.amount.clone() * BigDecimal::from(100))
        .to_u64()
        .ok_or_else(|| AppError::BadRequest("Invalid amount".into()))?;

    let reference = format!("WITH-{}", Uuid::new_v4());

    let transfer_req = InitiateTransferRequest {
        source: "balance".into(),
        amount: amount_kobo,
        recipient: bank_account.recipient_code,
        reason: payload.reason.clone(),
        reference: Some(reference.clone()),
    };

    let transfer_res = state.paystack_service.initiate_transfer(transfer_req).await?;

    // Dehydrate the result and update wallet balance immediately for safety (locking the funds)
    let transaction = conn.transaction::<_, AppError, _>(|c| {
        // Deduct balance
        diesel::update(wallets::table.filter(wallets::id.eq(wallet.id)))
            .set(wallets::balance.eq(wallets::balance - &payload.amount))
            .execute(c)?;

        // Create transaction record
        let tx = diesel::insert_into(wallet_transactions::table)
            .values(&NewWalletTransaction {
                wallet_id: wallet.id,
                amount: payload.amount,
                transaction_type: WalletTransactionTypeEnum::Withdrawal,
                status: WalletTransactionStatusEnum::Pending, // Transfers are usually async
                reference: reference,
                provider: "paystack".into(),
                description: payload.reason.or(Some("Wallet withdrawal".into())),
                metadata: Some(serde_json::json!({
                    "transfer_code": transfer_res.transfer_code,
                    "status": transfer_res.status
                })),
            })
            .get_result::<WalletTransaction>(c)?;
        
        Ok(tx)
    })?;

    Ok(ApiResponse::success_with_message("Withdrawal initiated successfully", transaction))
}

/// List wallet transaction history
///
/// Returns a paginated list of the authenticated user's wallet transactions.
/// Supports filtering by status, type, date range, amount range, and text search.
#[utoipa::path(
    get,
    path = "/api/wallets/transactions",
    params(TransactionFilters),
    responses(
        (status = 200, body = ApiResponse<TransactionListResponse>),
        (status = 401),
        (status = 500)
    ),
    tag = "wallets",
    security(("bearer_auth" = []))
)]
pub async fn list_transactions(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Query(filters): Query<TransactionFilters>,
) -> Result<ApiResponse<TransactionListResponse>, AppError> {
    let mut conn = state.pool.get()?;

    let wallet = get_or_create_wallet(&mut conn, current_user.id)?;

    let page = filters.page.max(1);
    let page_size = filters.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;

    // ---- Build base query ----
    let mut query = wallet_transactions::table
        .filter(wallet_transactions::wallet_id.eq(wallet.id))
        .into_boxed();

    // Status filter
    if let Some(status) = filters.status {
        query = query.filter(wallet_transactions::status.eq(status));
    }

    // Type filter
    if let Some(tx_type) = filters.transaction_type {
        query = query.filter(wallet_transactions::transaction_type.eq(tx_type));
    }

    // Date range filters — convert NaiveDate to start/end of day DateTime<Utc>
    if let Some(date_from) = filters.date_from {
        let start = date_from
            .and_hms_opt(0, 0, 0)
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .unwrap();
        query = query.filter(wallet_transactions::created_at.ge(start));
    }

    if let Some(date_to) = filters.date_to {
        let end = date_to
            .and_hms_opt(23, 59, 59)
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .unwrap();
        query = query.filter(wallet_transactions::created_at.le(end));
    }

    // Amount range filters — parse from string
    let amount_min = filters.amount_min
        .as_deref()
        .map(|s| s.parse::<BigDecimal>().ok())
        .flatten();

    let amount_max = filters.amount_max
        .as_deref()
        .map(|s| s.parse::<BigDecimal>().ok())
        .flatten();

    if let Some(ref min) = amount_min {
        query = query.filter(wallet_transactions::amount.ge(min.clone()));
    }

    if let Some(ref max) = amount_max {
        query = query.filter(wallet_transactions::amount.le(max.clone()));
    }

    // Text search — reference, description, or provider
    if let Some(ref search) = filters.search {
        let pattern = format!("%{}%", search.to_lowercase());
        query = query.filter(
            wallet_transactions::reference.ilike(pattern.clone())
                .or(wallet_transactions::description.ilike(pattern.clone()))
                .or(wallet_transactions::provider.ilike(pattern))
        );
    }

    // ---- Count total (clone the predicate) ----
    let mut count_query = wallet_transactions::table
        .filter(wallet_transactions::wallet_id.eq(wallet.id))
        .into_boxed();

    if let Some(status) = filters.status {
        count_query = count_query.filter(wallet_transactions::status.eq(status));
    }
    if let Some(tx_type) = filters.transaction_type {
        count_query = count_query.filter(wallet_transactions::transaction_type.eq(tx_type));
    }
    if let Some(date_from) = filters.date_from {
        let start = date_from.and_hms_opt(0, 0, 0)
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .unwrap();
        count_query = count_query.filter(wallet_transactions::created_at.ge(start));
    }
    if let Some(date_to) = filters.date_to {
        let end = date_to.and_hms_opt(23, 59, 59)
            .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
            .unwrap();
        count_query = count_query.filter(wallet_transactions::created_at.le(end));
    }
    if let Some(ref min) = amount_min {
        count_query = count_query.filter(wallet_transactions::amount.ge(min.clone()));
    }
    if let Some(ref max) = amount_max {
        count_query = count_query.filter(wallet_transactions::amount.le(max.clone()));
    }
    if let Some(ref search) = filters.search {
        let pattern = format!("%{}%", search.to_lowercase());
        count_query = count_query.filter(
            wallet_transactions::reference.ilike(pattern.clone())
                .or(wallet_transactions::description.ilike(pattern.clone()))
                .or(wallet_transactions::provider.ilike(pattern))
        );
    }

    let total: i64 = count_query
        .count()
        .get_result(&mut conn)?;

    let transactions = query
        .order(wallet_transactions::created_at.desc())
        .limit(page_size)
        .offset(offset)
        .load::<WalletTransaction>(&mut conn)?;

    let total_pages = (total + page_size - 1) / page_size;

    Ok(ApiResponse::success(TransactionListResponse {
        data: transactions,
        page,
        page_size,
        total,
        total_pages,
    }))
}

// Utility function to get or create a wallet for a user
pub fn get_or_create_wallet(conn: &mut PgConnection, user_id_val: Uuid) -> Result<Wallet, AppError> {
    let wallet = wallets::table
        .filter(wallets::user_id.eq(user_id_val))
        .first::<Wallet>(conn)
        .optional()?;

    match wallet {
        Some(w) => Ok(w),
        None => {
            let new_wallet = diesel::insert_into(wallets::table)
                .values((
                    wallets::user_id.eq(user_id_val),
                    wallets::balance.eq(BigDecimal::from(0)),
                    wallets::currency.eq("NGN"),
                ))
                .get_result::<Wallet>(conn)?;
            Ok(new_wallet)
        }
    }
}
