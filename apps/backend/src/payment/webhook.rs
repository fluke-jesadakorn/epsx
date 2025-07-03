use std::sync::Arc;
use crate::config::Config;
use crate::payment::service::{PaymentService, PaymentStatusUpdate, PaymentStatus};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sha2::{Sha256, Digest};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WebhookPayload {
    pub payment_id: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub network: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub timestamp: String,
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
    pub processed_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Payment not found")]
    PaymentNotFound,
    #[error("Invalid webhook payload: {0}")]
    InvalidPayload(String),
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),
}

pub struct WebhookHandler {
    payment_service: Arc<PaymentService>,
    config: Arc<Config>,
}

impl WebhookHandler {
    pub fn new(payment_service: Arc<PaymentService>, config: Arc<Config>) -> Self {
        Self {
            payment_service,
            config,
        }
    }

    pub async fn handle_payment_webhook(
        &self,
        payload: WebhookPayload,
    ) -> Result<WebhookResponse, WebhookError> {
        // Verify webhook signature
        self.verify_signature(&payload)?;

        // Parse status
        let status = match payload.status.as_str() {
            "completed" | "success" => PaymentStatus::Succeeded,
            "failed" | "error" => PaymentStatus::Failed,
            "pending" => PaymentStatus::Processing,
            "cancelled" => PaymentStatus::Cancelled,
            _ => return Err(WebhookError::InvalidPayload(format!("Unknown status: {}", payload.status))),
        };

        // Update payment status
        let update = PaymentStatusUpdate {
            payment_id: payload.payment_id.clone(),
            status,
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some({
                let mut metadata = std::collections::HashMap::new();
                if let Some(tx_hash) = payload.transaction_hash {
                    metadata.insert("transaction_hash".to_string(), tx_hash);
                }
                if let Some(network) = payload.network {
                    metadata.insert("network".to_string(), network);
                }
                metadata
            }),
        };

        match self.payment_service.update_payment_status(update).await {
            Ok(_) => Ok(WebhookResponse {
                success: true,
                message: "Payment status updated successfully".to_string(),
                processed_at: chrono::Utc::now().to_rfc3339(),
            }),
            Err(e) => Err(WebhookError::ProcessingFailed(e.to_string())),
        }
    }

    fn verify_signature(&self, payload: &WebhookPayload) -> Result<(), WebhookError> {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}", payload.payment_id, payload.status));
        let hash = hasher.finalize();
        let expected_signature = format!(
            "sha256={}",
            hex::encode(hash)
        );
        
        if payload.signature != expected_signature {
            return Err(WebhookError::InvalidSignature);
        }
        
        Ok(())
    }

    pub async fn handle_crypto_confirmation(
        &self,
        payment_id: &str,
        tx_hash: &str,
        network: &str,
        confirmations: u32,
    ) -> Result<WebhookResponse, WebhookError> {
        let required_confirmations = match network {
            "bitcoin" => 6,
            "ethereum" => 12,
            "tron" => 19,
            _ => 3,
        };

        let status = if confirmations >= required_confirmations {
            PaymentStatus::Succeeded
        } else {
            PaymentStatus::Processing
        };

        let update = PaymentStatusUpdate {
            payment_id: payment_id.to_string(),
            status,
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: Some({
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("transaction_hash".to_string(), tx_hash.to_string());
                metadata.insert("network".to_string(), network.to_string());
                metadata.insert("confirmations".to_string(), confirmations.to_string());
                metadata
            }),
        };

        match self.payment_service.update_payment_status(update).await {
            Ok(_) => Ok(WebhookResponse {
                success: true,
                message: format!("Payment confirmation updated: {}/{} confirmations", confirmations, required_confirmations),
                processed_at: chrono::Utc::now().to_rfc3339(),
            }),
            Err(e) => Err(WebhookError::ProcessingFailed(e.to_string())),
        }
    }
}

// Utility function for generating webhook URLs
pub fn generate_webhook_url(base_url: &str, payment_id: &str) -> String {
    format!("{}/api/webhooks/payment/{}", base_url, payment_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_signature_verification() {
        // Test implementation
        let config = Arc::new(Config::from_env());
        let payment_service = Arc::new(PaymentService::new(config.clone()));
        let handler = WebhookHandler::new(payment_service, config);

        let payload = WebhookPayload {
            payment_id: "test_payment_id".to_string(),
            status: "completed".to_string(),
            transaction_hash: Some("0x123456".to_string()),
            network: Some("ethereum".to_string()),
            amount: Some(100.0),
            currency: Some("USD".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            signature: "invalid_signature".to_string(),
        };

        let result = handler.handle_payment_webhook(payload).await;
        assert!(result.is_err());
    }
}
