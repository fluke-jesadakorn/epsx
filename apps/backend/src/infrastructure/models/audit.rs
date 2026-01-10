use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use crate::schemas::analytics::audit_logs;

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = audit_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuditLogDb {
    pub id: Uuid,
    pub wallet_address: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLogDb {
    pub wallet_address: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<JsonValue>,
}
