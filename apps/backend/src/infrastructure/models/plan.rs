//! Diesel Models for Groups
//!
//! Database models for groups table using Diesel ORM
//! (Previously groups - renamed for simplicity)

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for groups table
/// Note: Uses groups table until migration is run
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct PlanDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
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
}

/// Diesel Insertable model for creating new groups
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct NewPlanDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
    pub grace_period_hours: i32,
    pub rate_limit_per_minute: i32,
    pub rate_limit_per_hour: i32,
    pub rate_limit_per_day: i32,
    pub burst_capacity: i32,
    pub tier_level: i32,
    pub is_public: bool,
}

/// Diesel AsChangeset model for updating groups
#[derive(Debug, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::primary::plans)]
pub struct UpdatePlanDb {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub plan_type: Option<String>,
    pub plan_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_modified_by: Option<String>,
    pub grace_period_hours: Option<i32>,
    pub tier_level: Option<i32>,
    pub is_public: Option<bool>,
}

/// Form data for group creation from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: String,
    pub plan_type: String,
    pub plan_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

/// Form data for group updates from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePlanRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub plan_type: Option<String>,
    pub plan_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

// Type aliases for backward compatibility
pub type PermissionPlanDb = PlanDb;
pub type NewPermissionPlanDb = NewPlanDb;
pub type UpdatePermissionPlanDb = UpdatePlanDb;
pub type PermissionGroupDb = PlanDb;
pub type NewPermissionGroupDb = NewPlanDb;
pub type UpdatePermissionGroupDb = UpdatePlanDb;
pub type CreatePermissionGroupRequest = CreatePlanRequest;
pub type UpdatePermissionGroupRequest = UpdatePlanRequest;

/// Diesel Queryable model for plan permissions (junction table)
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schemas::primary::plan_permissions)]
pub struct PlanPermissionDb {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub permission_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
}

/// Diesel Insertable model for linking permissions to plans
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::primary::plan_permissions)]
pub struct NewPlanPermissionDb {
    pub plan_id: Uuid,
    pub permission_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
}
