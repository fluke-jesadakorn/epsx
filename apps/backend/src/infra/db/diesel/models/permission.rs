use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::{permission_profiles, temporary_permissions};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = permission_profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselPermissionProfile {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub permissions: Vec<String>,
    pub prerequisites: Vec<Uuid>,
    pub auto_assign: bool,
    pub auto_assign_conditions: Option<JsonValue>,
    pub expires_after_days: Option<i32>,
    pub max_assignments: Option<i32>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = permission_profiles)]
pub struct NewDieselPermissionProfile {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub permissions: Vec<String>,
    pub prerequisites: Vec<Uuid>,
    pub auto_assign: bool,
    pub auto_assign_conditions: Option<JsonValue>,
    pub expires_after_days: Option<i32>,
    pub max_assignments: Option<i32>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = permission_profiles)]
pub struct UpdateDieselPermissionProfile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub prerequisites: Option<Vec<Uuid>>,
    pub auto_assign: Option<bool>,
    pub auto_assign_conditions: Option<JsonValue>,
    pub expires_after_days: Option<i32>,
    pub max_assignments: Option<i32>,
    pub is_active: Option<bool>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = temporary_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselTemporaryPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub resource: Option<String>,
    pub action: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub granted_by: Uuid,
    pub reason: String,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = temporary_permissions)]
pub struct NewDieselTemporaryPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub resource: Option<String>,
    pub action: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub granted_by: Uuid,
    pub reason: String,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = temporary_permissions)]
pub struct UpdateDieselTemporaryPermission {
    pub status: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
    pub updated_at: DateTime<Utc>,
}