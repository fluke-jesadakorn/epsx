/**
 * Diesel Models for Permission Groups
 *
 * Database models for permission_groups table using Diesel ORM
 */

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

/// Diesel Queryable model for permission_groups table
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct PermissionGroupDb {
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

/// Diesel Insertable model for creating new permission groups
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct NewPermissionGroupDb {
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
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

/// Diesel AsChangeset model for updating permission groups
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::permission_groups)]
pub struct UpdatePermissionGroupDb {
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

/// Form data for permission group creation from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePermissionGroupRequest {
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

/// Form data for permission group updates from API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePermissionGroupRequest {
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