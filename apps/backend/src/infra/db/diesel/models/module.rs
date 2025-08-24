use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::{sub_modules, user_module_assignments, admin_modules, user_admin_roles};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = sub_modules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselSubModule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub quota_limits: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sub_modules)]
pub struct NewDieselSubModule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub quota_limits: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_module_assignments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUserModuleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub module_id: Uuid,
    pub access_level: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = user_module_assignments)]
pub struct NewDieselUserModuleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub module_id: Uuid,
    pub access_level: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = admin_modules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselAdminModule {
    pub id: Uuid,
    pub module_code: String,
    pub module_name: String,
    pub description: String,
    pub category: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub requires_modules: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = admin_modules)]
pub struct NewDieselAdminModule {
    pub id: Uuid,
    pub module_code: String,
    pub module_name: String,
    pub description: String,
    pub category: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
    pub requires_modules: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_admin_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUserAdminRole {
    pub id: Uuid,
    pub firebase_uid: String,
    pub module_code: String,
    pub granted_by: Option<String>,
    pub granted_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub assignment_metadata: Option<JsonValue>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = user_admin_roles)]
pub struct NewDieselUserAdminRole {
    pub id: Uuid,
    pub firebase_uid: String,
    pub module_code: String,
    pub granted_by: Option<String>,
    pub granted_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub assignment_metadata: Option<JsonValue>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}