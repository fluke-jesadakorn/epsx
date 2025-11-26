use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Payment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub cancellation_reason: Option<String>,
}

impl PaymentMetadata {
    pub fn new(created_at: DateTime<Utc>) -> Self {
        // Set expiry to 30 minutes from creation
        let expires_at = created_at + chrono::Duration::minutes(30);
        
        Self {
            created_at,
            updated_at: created_at,
            confirmed_at: None,
            completed_at: None,
            expires_at: Some(expires_at),
            failure_reason: None,
            cancellation_reason: None,
        }
    }

    pub fn mark_awaiting_payment(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn mark_confirmed(&mut self) {
        let now = Utc::now();
        self.confirmed_at = Some(now);
        self.updated_at = now;
    }

    pub fn mark_completed(&mut self, completed_at: DateTime<Utc>) {
        self.completed_at = Some(completed_at);
        self.updated_at = completed_at;
    }

    pub fn mark_failed(&mut self, failed_at: DateTime<Utc>, reason: String) {
        self.failure_reason = Some(reason);
        self.updated_at = failed_at;
    }

    pub fn mark_cancelled(&mut self, cancelled_at: DateTime<Utc>, reason: String) {
        self.cancellation_reason = Some(reason);
        self.updated_at = cancelled_at;
    }

    pub fn mark_refunded(&mut self, refunded_at: DateTime<Utc>) {
        self.updated_at = refunded_at;
    }
}