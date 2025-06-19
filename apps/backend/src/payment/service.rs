use std::sync::Arc;
use crate::config::Config;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(example = json!({
    "amount": 1000,
    "currency": "USD",
    "description": "Payment for subscription"
}))]
pub struct CreatePaymentRequest {
    pub amount: u64,
    pub currency: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaymentResponse {
    pub id: String,
    pub amount: u64,
    pub currency: String,
    pub status: PaymentStatus,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
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
        // Mock implementation for testing
        let payment_id = Uuid::new_v4().to_string();
        
        Ok(PaymentResponse {
            id: payment_id,
            amount: request.amount,
            currency: request.currency,
            status: PaymentStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
