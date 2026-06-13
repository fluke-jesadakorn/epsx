use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use crate::schemas::infra_logs::{audit_logs, unified_audit_log};

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

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = unified_audit_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UnifiedAuditDb {
    pub id: Uuid,
    pub actor: Option<String>,
    pub actor_type: String,
    pub created_at: DateTime<Utc>,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub effect: String,
    pub before_state: Option<JsonValue>,
    pub after_state: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<JsonValue>,
    pub category: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = unified_audit_log)]
pub struct NewUnifiedAuditDb {
    pub actor: Option<String>,
    pub actor_type: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub effect: String,
    pub before_state: Option<JsonValue>,
    pub after_state: Option<JsonValue>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<JsonValue>,
    pub category: String,
}
