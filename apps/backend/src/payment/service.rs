use std::sync::Arc;
use crate::config::Config;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "amount": 1000,
    "currency": "USD",
    "description": "Payment for subscription",
    "payment_method": "card",
    "metadata": {"user_id": "123"}
}))]
pub struct CreatePaymentRequest {
    pub amount: u64,
    pub currency: String,
    pub description: Option<String>,
    pub payment_method: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaymentResponse {
    pub id: String,
    pub amount: u64,
    pub currency: String,
    pub status: PaymentStatus,
    pub created_at: String,
    pub updated_at: String,
    pub expiration_date: String,
    pub user_level: UserLevel,
    pub qr_code: Option<String>,
    pub checkout_url: Option<String>,
    pub payment_method: String,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
    Expired,
    RequiresAction,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserLevel {
    Basic,
    Premium,
    VIP,
    Enterprise,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaymentStatusUpdate {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub updated_at: String,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaymentRetryRequest {
    pub payment_id: String,
    pub retry_reason: Option<String>,
}

#[derive(Debug, thiserror::Error, ToSchema)]
#[schema(example = json!({"error": "Invalid request: Amount must be greater than 0"}))]
#[allow(dead_code)]
pub enum PaymentError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Payment failed: {0}")]
    PaymentFailed(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PaymentService {
    config: Arc<Config>,
}

impl PaymentService {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<PaymentResponse, PaymentError> {
        // Validate amount
        if request.amount == 0 {
            return Err(PaymentError::InvalidRequest("Amount must be greater than 0".to_string()));
        }

        // Validate currency
        if request.currency.is_empty() {
            return Err(PaymentError::InvalidRequest("Currency is required".to_string()));
        }

        let payment_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let expiration = now + chrono::Duration::hours(24); // 24 hours expiration
        let payment_method = request.payment_method.unwrap_or_else(|| "crypto".to_string());

        // Generate QR code for crypto payments
        let qr_code = if payment_method == "crypto" || payment_method == "usdt" {
            Some(format!(
                "https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={}",
                format!("payment:{}:{}:{}", payment_id, request.amount, request.currency)
            ))
        } else {
            None
        };

        // Generate checkout URL for card payments
        let checkout_url = if payment_method == "card" {
            Some(format!("https://checkout.example.com/pay/{}", payment_id))
        } else {
            None
        };

        Ok(PaymentResponse {
            id: payment_id,
            amount: request.amount,
            currency: request.currency,
            status: PaymentStatus::Pending,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expiration_date: expiration.to_rfc3339(),
            user_level: UserLevel::Premium,
            qr_code,
            checkout_url,
            payment_method,
            retry_count: 0,
            error_message: None,
        })
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<PaymentResponse, PaymentError> {
        // Mock implementation - in real app, query database
        let now = chrono::Utc::now();
        
        Ok(PaymentResponse {
            id: payment_id.to_string(),
            amount: 1000,
            currency: "USD".to_string(),
            status: PaymentStatus::Processing,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expiration_date: (now + chrono::Duration::hours(24)).to_rfc3339(),
            user_level: UserLevel::Premium,
            qr_code: None,
            checkout_url: None,
            payment_method: "crypto".to_string(),
            retry_count: 0,
            error_message: None,
        })
    }

    pub async fn retry_payment(
        &self,
        request: PaymentRetryRequest,
    ) -> Result<PaymentResponse, PaymentError> {
        // Mock implementation - in real app, increment retry count and attempt payment again
        let now = chrono::Utc::now();
        
        Ok(PaymentResponse {
            id: request.payment_id,
            amount: 1000,
            currency: "USD".to_string(),
            status: PaymentStatus::Processing,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expiration_date: (now + chrono::Duration::hours(24)).to_rfc3339(),
            user_level: UserLevel::Premium,
            qr_code: None,
            checkout_url: None,
            payment_method: "crypto".to_string(),
            retry_count: 1,
            error_message: None,
        })
    }

    pub async fn update_payment_status(
        &self,
        update: PaymentStatusUpdate,
    ) -> Result<PaymentResponse, PaymentError> {
        // Mock implementation - in real app, update database
        let now = chrono::Utc::now();
        
        Ok(PaymentResponse {
            id: update.payment_id,
            amount: 1000,
            currency: "USD".to_string(),
            status: update.status,
            created_at: now.to_rfc3339(),
            updated_at: update.updated_at,
            expiration_date: (now + chrono::Duration::hours(24)).to_rfc3339(),
            user_level: UserLevel::Premium,
            qr_code: None,
            checkout_url: None,
            payment_method: "crypto".to_string(),
            retry_count: 0,
            error_message: None,
        })
    }

    pub async fn cancel_payment(
        &self,
        payment_id: &str,
    ) -> Result<PaymentResponse, PaymentError> {
        // Mock implementation - in real app, cancel payment in database
        let now = chrono::Utc::now();
        
        Ok(PaymentResponse {
            id: payment_id.to_string(),
            amount: 1000,
            currency: "USD".to_string(),
            status: PaymentStatus::Cancelled,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            expiration_date: (now + chrono::Duration::hours(24)).to_rfc3339(),
            user_level: UserLevel::Premium,
            qr_code: None,
            checkout_url: None,
            payment_method: "crypto".to_string(),
            retry_count: 0,
            error_message: Some("Payment cancelled by user".to_string()),
        })
    }
}
