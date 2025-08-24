use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::{iam_roles, iam_policies, iam_groups, user_permission_overrides};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = iam_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselIamRole {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policies: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = iam_roles)]
pub struct NewDieselIamRole {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policies: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = iam_policies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselIamPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub document: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = iam_policies)]
pub struct NewDieselIamPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub document: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = iam_groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselIamGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policies: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = iam_groups)]
pub struct NewDieselIamGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub policies: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_permission_overrides)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUserPermissionOverride {
    pub user_id: Uuid,
    pub allowed_permissions: Vec<String>,
    pub denied_permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = user_permission_overrides)]
pub struct NewDieselUserPermissionOverride {
    pub user_id: Uuid,
    pub allowed_permissions: Vec<String>,
    pub denied_permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}