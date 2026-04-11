use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::error::AppError;

pub struct PaystackService {
    secret_key: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
pub struct InitializeTransactionRequest {
    pub email: String,
    pub amount: u64, // amount in kobo/lowest unit (NGN 1.00 = 100 kobo)
    pub reference: Option<String>,
    pub callback_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct PaystackResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Deserialize, Debug)]
pub struct InitializeTransactionResponse {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyTransactionResponse {
    pub id: u64,
    pub domain: String,
    pub status: String,
    pub reference: String,
    pub amount: u64,
    pub gateway_response: String,
    pub channel: String,
    pub currency: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Bank {
    pub name: String,
    pub slug: String,
    pub code: String,
    pub active: bool,
}

#[derive(Deserialize, Debug)]
pub struct ResolveAccountResponse {
    pub account_number: String,
    pub account_name: String,
    pub bank_id: u32,
}

#[derive(Serialize)]
pub struct CreateTransferRecipientRequest {
    pub r#type: String, // nubean
    pub name: String,
    pub account_number: String,
    pub bank_code: String,
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct TransferRecipientResponse {
    pub active: bool,
    pub recipient_code: String,
    pub name: String,
    pub details: serde_json::Value,
}

#[derive(Serialize)]
pub struct InitiateTransferRequest {
    pub source: String, // balance
    pub amount: u64,
    pub recipient: String,
    pub reason: Option<String>,
    pub reference: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct InitiateTransferResponse {
    pub reference: String,
    pub status: String,
    pub amount: u64,
    pub transfer_code: String,
}

impl PaystackService {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.secret_key)).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn initialize_transaction(
        &self,
        req: InitializeTransactionRequest,
    ) -> Result<InitializeTransactionResponse, AppError> {
        let url = "https://api.paystack.co/transaction/initialize";
        let res = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&req)
            .send()
            .await?
            .json::<PaystackResponse<InitializeTransactionResponse>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }

    pub async fn verify_transaction(
        &self,
        reference: &str,
    ) -> Result<VerifyTransactionResponse, AppError> {
        let url = format!("https://api.paystack.co/transaction/verify/{}", reference);
        let res = self
            .client
            .get(url)
            .headers(self.headers())
            .send()
            .await?
            .json::<PaystackResponse<VerifyTransactionResponse>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }

    pub async fn list_banks(&self) -> Result<Vec<Bank>, AppError> {
        let url = "https://api.paystack.co/bank?currency=NGN";
        let res = self
            .client
            .get(url)
            .headers(self.headers())
            .send()
            .await?
            .json::<PaystackResponse<Vec<Bank>>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }

    pub async fn resolve_account(
        &self,
        account_number: &str,
        bank_code: &str,
    ) -> Result<ResolveAccountResponse, AppError> {
        let url = format!(
            "https://api.paystack.co/bank/resolve?account_number={}&bank_code={}",
            account_number, bank_code
        );
        let res = self
            .client
            .get(url)
            .headers(self.headers())
            .send()
            .await?
            .json::<PaystackResponse<ResolveAccountResponse>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }

    pub async fn create_transfer_recipient(
        &self,
        req: CreateTransferRecipientRequest,
    ) -> Result<TransferRecipientResponse, AppError> {
        let url = "https://api.paystack.co/transferrecipient";
        let res = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&req)
            .send()
            .await?
            .json::<PaystackResponse<TransferRecipientResponse>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }

    pub async fn initiate_transfer(
        &self,
        req: InitiateTransferRequest,
    ) -> Result<InitiateTransferResponse, AppError> {
        let url = "https://api.paystack.co/transfer";
        let res = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&req)
            .send()
            .await?
            .json::<PaystackResponse<InitiateTransferResponse>>()
            .await?;

        if !res.status {
            return Err(AppError::BadRequest(res.message));
        }

        res.data.ok_or_else(|| AppError::InternalServerError)
    }
}
