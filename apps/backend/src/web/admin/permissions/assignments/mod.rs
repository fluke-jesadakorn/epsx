// Wallet-Plan Assignment Management
// Consolidates assignment operations from permission_plan_handlers.rs and normalized_permission_handlers.rs

pub mod create;
pub mod remove;
pub mod queries;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub wallet_address: String,
    pub plan_id: String,
    pub assignment_source: String, // "manual" | "payment" | "web3_asset" | "dao_governance" | "admin" | "migration" | "auto_assignment"
    pub assignment_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub assignment_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssignmentResponse {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: String,
    pub plan_name: String,
    pub plan_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub assignment_reason: Option<String>,
    pub assigned_by: Option<String>,
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub auto_renew: bool,
    pub next_billing_date: Option<DateTime<Utc>>,
    pub assignment_metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub wallet_address: Option<String>,
    pub plan_id: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExpiringAssignmentsQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PlanHistoryQuery {
    pub operation_type: Option<String>,
    pub operation_source: Option<String>,
    pub plan_id: Option<String>,
    pub user_search: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PlanHistoryResponse {
    pub id: String,
    pub user_id: String,
    pub user_email: Option<String>, // Not available in blockchain auth, but kept for interface compat
    pub user_name: Option<String>,
    pub plan_id: String,
    pub plan_name: Option<String>,
    pub operation_type: String,
    pub operation_source: String,
    pub performed_by: Option<String>,
    pub performed_by_name: Option<String>,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Re-export handlers
pub use create::create_assignment;
pub use remove::remove_assignment;
pub use queries::{list_assignments, get_expiring_assignments, get_assignment_history, get_wallet_plans, get_plan_history};
