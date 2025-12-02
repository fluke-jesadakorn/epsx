/**
 * Diesel Models for Payments and Subscriptions
 *
 * Database models for payments, subscriptions, and payment_audit_log tables using Diesel ORM
 */

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset, QueryableByName};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for payments table
#[derive(Debug, Clone, Queryable)]
#[diesel(table_name = crate::schema::payments)]
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
    pub metadata: serde_json::Value,
}

/// Diesel Insertable model for creating new payments
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::payments)]
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

/// Diesel AsChangeset model for updating payments
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::payments)]
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

/// Diesel Queryable model for subscriptions table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::subscriptions)]
pub struct SubscriptionDb {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Diesel Insertable model for creating new subscriptions
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::subscriptions)]
pub struct NewSubscriptionDb {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Uuid,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Diesel AsChangeset model for updating subscriptions
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::subscriptions)]
pub struct UpdateSubscriptionDb {
    pub status: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Diesel Queryable model for payment_audit_log table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::payment_audit_log)]
pub struct PaymentAuditLogDb {
    pub id: Uuid,
    pub payment_id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Diesel Insertable model for creating audit log entries
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::payment_audit_log)]
pub struct NewPaymentAuditLogDb {
    pub payment_id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub metadata: serde_json::Value,
}

/// Form data for payment creation from API requests
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

/// Form data for payment updates from API requests
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

/// Form data for subscription creation from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSubscriptionRequest {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub payment_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Form data for subscription updates from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateSubscriptionRequest {
    pub status: Option<String>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Payment statistics aggregation result
#[derive(Debug, Clone)]
pub struct PaymentStatsDb {
    pub total_payments: i64,
    pub completed_payments: i64,
    pub failed_payments: i64,
    pub total_amount: BigDecimal,
    pub average_amount: BigDecimal,
    pub last_payment_date: Option<DateTime<Utc>>,
}

/// Payment summary for admin dashboard
#[derive(Debug, Clone, Queryable, Selectable, QueryableByName)]
#[diesel(table_name = crate::schema::payments)]
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