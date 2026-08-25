//! Database models for payments and subscriptions (sqlx-friendly).
//!
//! BIG-BANG: migrated from diesel to plain sqlx structs.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Row model for payments table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentDb {
    pub id: Uuid,
    pub payment_reference: String,
    pub transaction_hash: Option<String>,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub plan_id: Uuid,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub network: Option<String>,
}

/// Insert model for payments.
#[derive(Debug, Clone, Deserialize)]
pub struct NewPaymentDb {
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub method: String,
    pub status: String,
    pub plan_id: Uuid,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Update model for payments.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdatePaymentDb {
    pub transaction_hash: Option<String>,
    pub status: Option<String>,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: Option<i32>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Row model for subscriptions table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionDb {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Insert model for subscriptions.
#[derive(Debug, Clone, Deserialize)]
pub struct NewSubscriptionDb {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Update model for subscriptions.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateSubscriptionDb {
    pub status: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Row model for payment_audit_log table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentAuditLogDb {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Insert model for payment_audit_log.
#[derive(Debug, Clone, Deserialize)]
pub struct NewPaymentAuditLogDb {
    pub payment_id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub metadata: serde_json::Value,
}

/// Form data for payment creation.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePaymentRequest {
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub method: String,
    pub plan_id: Uuid,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Form data for payment updates.
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePaymentRequest {
    pub transaction_hash: Option<String>,
    pub status: Option<String>,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: Option<i32>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Form data for subscription creation.
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSubscriptionRequest {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Form data for subscription updates.
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateSubscriptionRequest {
    pub status: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Payment statistics aggregation result.
#[derive(Debug, Clone)]
pub struct PaymentStatsDb {
    pub total_payments: i64,
    pub completed_payments: i64,
    pub failed_payments: i64,
    pub total_amount: BigDecimal,
    pub average_amount: BigDecimal,
    pub last_payment_date: Option<DateTime<Utc>>,
}

/// Payment summary for admin dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentSummaryDb {
    pub id: Uuid,
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: BigDecimal,
    pub currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}