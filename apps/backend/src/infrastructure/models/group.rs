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
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::groups)]
pub struct GroupDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// Diesel Insertable model for creating new groups
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::groups)]
pub struct NewGroupDb {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub group_metadata: serde_json::Value,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// Diesel AsChangeset model for updating groups
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::groups)]
pub struct UpdateGroupDb {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub group_type: Option<String>,
    pub group_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_modified_by: Option<String>,
}

/// Form data for group creation from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: String,
    pub group_type: String,
    pub group_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
}

/// Form data for group updates from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub group_type: Option<String>,
    pub group_metadata: Option<serde_json::Value>,
    pub price: Option<BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: Option<serde_json::Value>,
}

// Type aliases for backward compatibility
pub type PermissionGroupDb = GroupDb;
pub type NewPermissionGroupDb = NewGroupDb;
pub type UpdatePermissionGroupDb = UpdateGroupDb;
pub type CreatePermissionGroupRequest = CreateGroupRequest;
pub type UpdatePermissionGroupRequest = UpdateGroupRequest;

/// Diesel Queryable model for group permissions (junction table)
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::group_permissions)]
pub struct GroupPermissionDb {
    pub id: Uuid,
    pub group_id: Uuid,
    pub permission_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
}

/// Diesel Insertable model for linking permissions to groups
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::group_permissions)]
pub struct NewGroupPermissionDb {
    pub group_id: Uuid,
    pub permission_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
}
