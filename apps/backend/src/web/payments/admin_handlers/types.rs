//! Admin Payment Management Types
//!
//! DTOs and structs for admin payment operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Admin payment list query parameters
#[derive(Debug, Deserialize)]
pub struct AdminPaymentListParams {
    /// Page number for pagination
    pub page: Option<u32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Filter by payment status
    pub status: Option<String>,
    /// Filter by wallet address
    pub wallet_address: Option<String>,
    /// Filter by plan ID
    pub plan_id: Option<Uuid>,
    /// Filter by date range (start)
    pub start_date: Option<String>,
    /// Filter by date range (end)
    pub end_date: Option<String>,
    /// Search by transaction hash or reference
    pub search: Option<String>,
}

/// Admin payment list response
#[derive(Debug, Serialize)]
pub struct AdminPaymentListResponse {
    pub success: bool,
    pub payments: Vec<AdminPaymentInfo>,
    pub pagination: PaginationInfo,
    pub summary: PaymentSummary,
}

/// Admin payment information
#[derive(Debug, Serialize)]
pub struct AdminPaymentInfo {
    pub id: Uuid,
    pub payment_reference: String,
    pub wallet_address: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub transaction_hash: Option<String>,
    pub contract_address: Option<String>,
    pub token_address: Option<String>,
    pub block_number: Option<i64>,
    pub confirmations: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl AdminPaymentInfo {
    /// Create from PaymentDb with plan name
    pub fn from_db(pay: crate::infrastructure::models::payment::PaymentDb, plan_name: String) -> Self {
        Self {
            id: pay.id,
            payment_reference: pay.payment_reference,
            wallet_address: pay.wallet_address,
            amount: pay.amount.to_string().parse::<f64>().unwrap_or(0.0),
            currency: pay.currency,
            status: pay.status,
            plan_id: pay.plan_id,
            plan_name,
            transaction_hash: pay.transaction_hash,
            contract_address: pay.contract_address,
            token_address: pay.token_address,
            block_number: pay.block_number,
            confirmations: pay.confirmations.unwrap_or(0),
            created_at: pay.created_at.unwrap_or_else(Utc::now),
            updated_at: pay.updated_at.unwrap_or_else(Utc::now),
            completed_at: pay.completed_at,
            expires_at: pay.expires_at,
            metadata: pay.metadata.unwrap_or(serde_json::json!({})),
        }
    }
}

/// Pagination information
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total_count: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Payment summary statistics
#[derive(Debug, Serialize)]
pub struct PaymentSummary {
    pub total_payments: u64,
    pub total_amount: f64,
    pub successful_payments: u64,
    pub failed_payments: u64,
    pub pending_payments: u64,
    pub average_payment_amount: f64,
    pub payments_today: u64,
    pub revenue_today: f64,
}

/// Admin payment details response
#[derive(Debug, Serialize)]
pub struct AdminPaymentDetailsResponse {
    pub success: bool,
    pub payment: Option<AdminPaymentInfo>,
    pub audit_logs: Vec<PaymentAuditLog>,
}

/// Payment audit log entry
#[derive(Debug, Serialize)]
pub struct PaymentAuditLog {
    pub id: Uuid,
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Admin subscription list response
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionListResponse {
    pub success: bool,
    pub subscriptions: Vec<AdminSubscriptionInfo>,
    pub pagination: PaginationInfo,
    pub summary: SubscriptionSummary,
}

/// Admin subscription information
#[derive(Debug, Serialize)]
pub struct AdminSubscriptionInfo {
    pub id: Uuid,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub status: String,
    pub payment_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: serde_json::Value,
}

/// Subscription summary statistics
#[derive(Debug, Serialize)]
pub struct SubscriptionSummary {
    pub total_subscriptions: u64,
    pub active_subscriptions: u64,
    pub expired_subscriptions: u64,
    pub cancelled_subscriptions: u64,
    pub new_subscriptions_today: u64,
    pub expiring_soon: u64, // Subscriptions expiring in next 7 days
    pub monthly_revenue: f64,
}

/// Payment analytics response
#[derive(Debug, Serialize)]
pub struct PaymentAnalyticsResponse {
    pub success: bool,
    pub analytics: PaymentAnalytics,
}

/// Payment analytics data
#[derive(Debug, Serialize)]
pub struct PaymentAnalytics {
    pub daily_revenue: Vec<DailyRevenue>,
    pub plan_breakdown: Vec<PlanBreakdown>,
    pub payment_methods: Vec<PaymentMethodStats>,
    pub trends: PaymentTrends,
}

/// Daily revenue data
#[derive(Debug, Serialize)]
pub struct DailyRevenue {
    pub date: String,
    pub revenue: f64,
    pub payment_count: u32,
}

/// Plan breakdown data
#[derive(Debug, Serialize)]
pub struct PlanBreakdown {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub subscription_count: u32,
    pub revenue: f64,
    pub average_revenue_per_user: f64,
}

/// Payment method statistics
#[derive(Debug, Serialize)]
pub struct PaymentMethodStats {
    pub method: String,
    pub count: u32,
    pub revenue: f64,
    pub success_rate: f64,
}

/// Payment trends
#[derive(Debug, Serialize)]
pub struct PaymentTrends {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub average_subscription_length: f64,
    pub customer_lifetime_value: f64,
}

/// Refund payment request
#[derive(Debug, Deserialize)]
pub struct RefundPaymentRequest {
    pub reason: String,
    pub refund_amount: Option<f64>,
    pub partial_refund: bool,
    pub notify_user: bool,
}

/// Refund payment response
#[derive(Debug, Serialize)]
pub struct RefundPaymentResponse {
    pub success: bool,
    pub message: String,
    pub refund_id: Option<String>,
    pub refund_amount: f64,
    pub processed_at: DateTime<Utc>,
}

/// Update payment status request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub status: String,
    pub reason: Option<String>,
    pub notify_user: bool,
    pub metadata: Option<serde_json::Value>,
}

/// Update payment status response
#[derive(Debug, Serialize)]
pub struct UpdatePaymentStatusResponse {
    pub success: bool,
    pub message: String,
    pub old_status: String,
    pub new_status: String,
    pub updated_at: DateTime<Utc>,
}
