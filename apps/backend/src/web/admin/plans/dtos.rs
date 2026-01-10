use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlanRequest {
    #[schema(example = "Advanced Plan")]
    pub name: String,
    #[schema(example = "Plan for advanced users")]
    pub description: Option<String>,
    #[schema(example = "Advanced Access Group")]
    pub permission_group_name: String, 
    #[schema(value_type = String, example = "29.99")]
    pub current_price: Decimal,
    #[schema(example = "USD")]
    pub currency: String,
    #[schema(example = "traders")]
    pub target_audience: String,
    #[schema(example = "monthly")]
    pub billing_model: String,
    #[schema(example = json!(["epsx:api:calls:1000"]))]
    pub permissions: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub tier_level: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[schema(value_type = String)]
    pub current_price: Option<Decimal>,
    pub is_active: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub tier_level: Option<i32>,
    pub billing_model: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlanResponse {
    pub id: String, // UUID
    pub name: String,
    pub description: Option<String>,
    pub permission_group_name: String,
    #[schema(value_type = String)]
    pub current_price: Decimal,
    pub effective_price: f64,
    pub promotion_active: bool,
    pub promotion_status: String,
    pub promotion_discount: f64,
    pub currency: String,
    pub target_audience: String,
    pub billing_model: String,
    pub group_type: String,
    pub plan_category: String,
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub subscriber_count: u64,
    #[schema(value_type = String)]
    pub revenue_last_30_days: Decimal,
    pub tier_level: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlanListResponse {
    pub success: bool,
    pub data: PlanListData,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlanListData {
    pub plans: Vec<PlanResponse>,
    pub total_count: usize,
    pub has_more: bool,
}

// Subscription DTOs
#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub permission_group_name: String, // e.g., "Premium Access Group"
    pub access_context: String,
    pub api_key_name: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub permission_group_name: String,
    pub permissions_granted: Vec<String>,
    pub group_type: String,
    pub access_context: String,
    pub api_key: Option<String>,
    pub api_key_name: Option<String>,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub current_usage: serde_json::Value,
    pub quota_limits: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UserAccessListQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>, // "active", "expired", "expiring_soon", "no_plan"
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserAccessData {
    pub wallet_address: String,
    pub current_plan_id: Option<Uuid>,
    pub plan_name: Option<String>,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub days_remaining: i64,
    pub status: String, // "active", "expiring_soon", "expired", "no_plan"
}

// Permissions Group DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct PermissionGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub group_type: String, 
}
