use std::sync::Arc;
use crate::config::Config;
use crate::payment::service::{PaymentService, PaymentStatusUpdate, PaymentStatus};
use crate::dom::services::auto_assignment::AutoAssignmentEngine;
use crate::dom::entities::permission_profile::PermissionProfileId;
use crate::dom::values::UserId;
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
    // Enhanced fields for permission profile auto-assignment
    pub user_id: Option<String>,
    pub permission_profile_id: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
    pub processed_at: String,
    // Enhanced fields for auto-assignment feedback
    pub features_activated: Option<bool>,
    pub permission_profile_assigned: Option<String>,
    pub assignment_details: Option<AssignmentDetails>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AssignmentDetails {
    pub permission_profile_id: String,
    pub permission_profile_name: String,
    pub features_unlocked: Vec<String>,
    pub expires_at: Option<String>,
    pub activated_at: String,
}

#[derive(Debug, Clone)]
pub struct PaymentAssignmentContext {
    pub payment_id: String,
    pub amount: Option<f64>,
    pub currency: String,
    pub network: Option<String>,
    pub transaction_hash: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PaymentAssignmentResult {
    pub permission_profile_name: Option<String>,
    pub features_unlocked: Vec<String>,
    pub assignment_id: String,
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
    auto_assignment_engine: Arc<AutoAssignmentEngine>,
    config: Arc<Config>,
}

impl WebhookHandler {
    pub fn new(
        payment_service: Arc<PaymentService>, 
        auto_assignment_engine: Arc<AutoAssignmentEngine>,
        config: Arc<Config>
    ) -> Self {
        Self {
            payment_service,
            auto_assignment_engine,
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
                // Add payment metadata if provided
                if let Some(ref payload_metadata) = payload.metadata {
                    for (key, value) in payload_metadata {
                        metadata.insert(key.clone(), value.clone());
                    }
                }
                metadata
            }),
        };

        // Update payment status first
        self.payment_service.update_payment_status(update).await
            .map_err(|e| WebhookError::ProcessingFailed(e.to_string()))?;

        // Process auto-assignment if payment succeeded and we have the required data
        let assignment_result = if status == PaymentStatus::Succeeded {
            self.process_payment_auto_assignment(&payload).await
        } else {
            None
        };

        // Build response with assignment details
        let response = WebhookResponse {
            success: true,
            message: if assignment_result.is_some() {
                "Payment processed and features activated successfully".to_string()
            } else {
                "Payment status updated successfully".to_string()
            },
            processed_at: chrono::Utc::now().to_rfc3339(),
            features_activated: assignment_result.as_ref().map(|_| true),
            permission_profile_assigned: assignment_result.as_ref().map(|r| r.permission_profile_name.clone()),
            assignment_details: assignment_result,
        };

        tracing::info!(
            "Payment webhook processed: {} - Status: {} - Features activated: {}",
            payload.payment_id,
            payload.status,
            response.features_activated.unwrap_or(false)
        );

        Ok(response)
    }

    /// Process auto-assignment for successful payments
    async fn process_payment_auto_assignment(
        &self,
        payload: &WebhookPayload,
    ) -> Option<AssignmentDetails> {
        // Check if we have required data for auto-assignment
        let user_id = payload.user_id.as_ref()?;
        let permission_profile_id = payload.permission_profile_id.as_ref()?;

        // Parse IDs
        let user_id = match UserId::new(user_id.clone()) {
            user_id => user_id,
        };
        let permission_profile_id = match PermissionProfileId::new(permission_profile_id.clone()) {
            permission_profile_id => permission_profile_id,
        };

        // Process auto-assignment
        match self.process_permission_profile_assignment(&user_id, &permission_profile_id, payload).await {
            Ok(details) => {
                tracing::info!(
                    "Auto-assignment successful: User {} assigned permission profile {} via payment {}",
                    user_id, permission_profile_id, payload.payment_id
                );
                Some(details)
            }
            Err(e) => {
                tracing::error!(
                    "Auto-assignment failed for payment {}: {}",
                    payload.payment_id, e
                );
                None
            }
        }
    }

    /// Process permission profile assignment for payment
    async fn process_permission_profile_assignment(
        &self,
        user_id: &UserId,
        permission_profile_id: &PermissionProfileId,
        payload: &WebhookPayload,
    ) -> Result<AssignmentDetails, String> {
        // Create assignment context from payment data
        let context = self.create_payment_assignment_context(payload);

        // Use auto-assignment engine to process the assignment
        let assignment_result = self.auto_assignment_engine
            .process_payment_completion(
                &payload.payment_id,
                user_id,
                permission_profile_id,
                &context,
            )
            .await
            .map_err(|e| format!("Assignment failed: {}", e))?;

        // Calculate expiration based on payment amount or metadata
        let expires_at = self.calculate_feature_expiration(payload);

        Ok(AssignmentDetails {
            permission_profile_id: permission_profile_id.value().to_string(),
            permission_profile_name: assignment_result.permission_profile_name.unwrap_or_else(|| "Unknown".to_string()),
            features_unlocked: assignment_result.features_unlocked,
            expires_at: expires_at.map(|dt| dt.to_rfc3339()),
            activated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Create assignment context from payment webhook data
    fn create_payment_assignment_context(&self, payload: &WebhookPayload) -> PaymentAssignmentContext {
        PaymentAssignmentContext {
            payment_id: payload.payment_id.clone(),
            amount: payload.amount,
            currency: payload.currency.clone().unwrap_or_else(|| "USD".to_string()),
            network: payload.network.clone(),
            transaction_hash: payload.transaction_hash.clone(),
            metadata: payload.metadata.clone().unwrap_or_default(),
            processed_at: chrono::Utc::now(),
        }
    }

    /// Calculate feature expiration based on payment data
    fn calculate_feature_expiration(&self, payload: &WebhookPayload) -> Option<chrono::DateTime<chrono::Utc>> {
        // Check metadata for expiration information
        if let Some(ref metadata) = payload.metadata {
            if let Some(duration) = metadata.get("subscription_duration") {
                return match duration.as_str() {
                    "monthly" => Some(chrono::Utc::now() + chrono::Duration::days(30)),
                    "quarterly" => Some(chrono::Utc::now() + chrono::Duration::days(90)),
                    "yearly" => Some(chrono::Utc::now() + chrono::Duration::days(365)),
                    "lifetime" => None, // No expiration for lifetime purchases
                    _ => Some(chrono::Utc::now() + chrono::Duration::days(30)), // Default to monthly
                };
            }
        }

        // Fallback: calculate based on amount (example logic)
        if let Some(amount) = payload.amount {
            if amount >= 100.0 {
                Some(chrono::Utc::now() + chrono::Duration::days(365)) // Yearly for $100+
            } else if amount >= 30.0 {
                Some(chrono::Utc::now() + chrono::Duration::days(90)) // Quarterly for $30+
            } else {
                Some(chrono::Utc::now() + chrono::Duration::days(30)) // Monthly for less
            }
        } else {
            Some(chrono::Utc::now() + chrono::Duration::days(30)) // Default monthly
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
