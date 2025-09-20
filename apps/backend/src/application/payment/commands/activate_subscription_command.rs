use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::payment::value_objects::{PaymentId, TransactionHash};

/// Command to activate user subscription after successful payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateSubscriptionCommand {
    pub payment_id: PaymentId,
    pub user_id: UserId,
    pub plan_id: i32,
    pub transaction_hash: TransactionHash,
    pub confirmed_at: DateTime<Utc>,
}

impl ActivateSubscriptionCommand {
    pub fn new(
        payment_id: PaymentId,
        user_id: UserId,
        plan_id: i32,
        transaction_hash: TransactionHash,
    ) -> Self {
        Self {
            payment_id,
            user_id,
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
    pub user_id: UserId,
    pub plan_id: i32,
    pub plan_name: String,
    pub activated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub transaction_hash: String,
    pub error_message: Option<String>,
}

impl SubscriptionActivationResult {
    pub fn success(
        user_id: UserId,
        plan_id: i32,
        plan_name: String,
        transaction_hash: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            success: true,
            user_id,
            plan_id,
            plan_name,
            activated_at: Utc::now(),
            expires_at,
            transaction_hash,
            error_message: None,
        }
    }

    pub fn failure(
        user_id: UserId,
        plan_id: i32,
        transaction_hash: String,
        error: String,
    ) -> Self {
        Self {
            success: false,
            user_id,
            plan_id,
            plan_name: "Unknown".to_string(),
            activated_at: Utc::now(),
            expires_at: None,
            transaction_hash,
            error_message: Some(error),
        }
    }
}