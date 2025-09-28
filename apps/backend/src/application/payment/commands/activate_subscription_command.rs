use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::payment::value_objects::{PaymentId, TransactionHash};

/// Command to activate user subscription after successful payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateSubscriptionCommand {
    pub payment_id: PaymentId,
    pub wallet_address: UserId,
    pub plan_id: i32,
    pub transaction_hash: TransactionHash,
    pub confirmed_at: DateTime<Utc>,
}

impl ActivateSubscriptionCommand {
    pub fn new(
        payment_id: PaymentId,
        wallet_address: UserId,
        plan_id: i32,
        transaction_hash: TransactionHash,
    ) -> Self {
        Self {
            payment_id,
            wallet_address,
            plan_id,
            transaction_hash,
            confirmed_at: Utc::now(),
        }
    }
}

/// Result of subscription activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionActivationResult {
    pub success: bool,
    pub wallet_address: UserId,
    pub plan_id: i32,
    pub plan_name: String,
    pub activated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub transaction_hash: String,
    pub error_message: Option<String>,
}

impl SubscriptionActivationResult {
    pub fn success(
        wallet_address: UserId,
        plan_id: i32,
        plan_name: String,
        transaction_hash: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            success: true,
            wallet_address,
            plan_id,
            plan_name,
            activated_at: Utc::now(),
            expires_at,
            transaction_hash,
            error_message: None,
        }
    }

    pub fn failure(
        wallet_address: UserId,
        plan_id: i32,
        transaction_hash: String,
        error: String,
    ) -> Self {
        Self {
            success: false,
            wallet_address,
            plan_id,
            plan_name: "Unknown".to_string(),
            activated_at: Utc::now(),
            expires_at: None,
            transaction_hash,
            error_message: Some(error),
        }
    }
}