//! Models for plans table
//!
//! BIG-BANG: migrated to sqlx::FromRow (real).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// PlanDb for sqlx — represents the `plans` table row.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlanDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
    pub grace_period_hours: i32,
    pub tier_level: i32,
    pub is_public: bool,
    pub plan_category: String,
    pub plan_group: String,
    pub is_system: bool,
}

/// NewPlanDb for inserts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPlanDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub grace_period_hours: i32,
    pub tier_level: i32,
    pub is_public: bool,
    pub plan_category: String,
    pub plan_group: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// UpdatePlanDb for partial updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanDb {
    pub name: Option<String>,
    pub description: Option<String>,
    pub plan_metadata: Option<serde_json::Value>,
    pub price: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub tier_level: Option<i32>,
    pub is_public: Option<bool>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub burst_capacity: Option<i32>,
}
