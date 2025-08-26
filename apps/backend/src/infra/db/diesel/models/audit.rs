use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::audit_logs;
use crate::infra::db::diesel::types::DieselIpAddr;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = audit_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselAuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: Option<String>,
    pub severity: Option<String>,
    pub details: Option<JsonValue>,
    pub ip_address: Option<DieselIpAddr>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<Uuid>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = audit_logs)]
pub struct NewDieselAuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: Option<String>,
    pub severity: Option<String>,
    pub details: Option<JsonValue>,
    pub ip_address: Option<DieselIpAddr>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<Uuid>,
}