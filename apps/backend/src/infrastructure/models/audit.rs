use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
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

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "audit_log_new")]
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

#[derive(Debug, Clone, sqlx::FromRow)]
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

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "unified_audit_log_new")]
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
