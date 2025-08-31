use uuid::Uuid;

use crate::dom::entities::audit::AuditLogEntry;
use crate::dom::entities::audit::AuditLogId;
use crate::dom::values::UserId;
use crate::infra::db::diesel::models::{DieselAuditLog, NewDieselAuditLog};
use crate::dom::entities::audit::AuditError;

impl TryFrom<DieselAuditLog> for AuditLogEntry {
    type Error = AuditError;

    fn try_from(diesel_audit: DieselAuditLog) -> Result<Self, Self::Error> {
        let audit_id = AuditLogId::from_str(&diesel_audit.id.to_string())
            .map_err(|e| AuditError::SerializationError(format!("Invalid AuditLogId: {}", e)))?;
        
        let user_id = diesel_audit.user_id
            .map(|id| UserId::from_str(&id.to_string()))
            .transpose()
            .map_err(|e| AuditError::SerializationError(format!("Invalid UserId: {}", e)))?;
        
        Ok(AuditLogEntry::reconstruct(
            audit_id,
            user_id.unwrap_or_else(|| UserId::generate()),
            crate::dom::entities::audit::AuditAction::UserCreated,
            crate::dom::entities::audit::ResourceType::User, 
            diesel_audit.resource_id.unwrap_or_default(),
            crate::dom::entities::audit::AuditResult::Success,
            crate::dom::entities::audit::AuditMetadata {
                previous_values: None,
                new_values: None,
                error_message: None,
                additional_data: Default::default(),
                affected_count: None,
                duration_ms: None,
            },
            diesel_audit.timestamp,
            diesel_audit.ip_address.map(|ip| ip.to_string()),
            diesel_audit.user_agent,
            None, // session_id
        ))
    }
}

impl TryFrom<&AuditLogEntry> for NewDieselAuditLog {
    type Error = AuditError;

    fn try_from(entry: &AuditLogEntry) -> Result<Self, Self::Error> {
        let audit_uuid = Uuid::parse_str(&entry.id().to_string())
            .map_err(|e| AuditError::SerializationError(format!("Invalid audit UUID: {}", e)))?;
        
        let user_uuid = Uuid::parse_str(&entry.user_id().to_string())
            .map_err(|e| AuditError::SerializationError(format!("Invalid user UUID: {}", e)))?;
        
        let ip_address = entry.ip_address()
            .as_ref()
            .map(|ip| ip.parse())
            .transpose()
            .map_err(|e| AuditError::SerializationError(format!("Invalid IP address: {}", e)))?;
        
        Ok(NewDieselAuditLog {
            id: audit_uuid,
            actor_id: Some(user_uuid),
            user_id: Some(user_uuid),
            action: entry.action().to_string(),
            resource_type: entry.resource_type().to_string(),
            resource_id: Some(entry.resource_id().to_string()),
            result: None,
            severity: None,
            details: Some(serde_json::to_value(entry.metadata()).unwrap_or_default()),
            ip_address,
            user_agent: entry.user_agent().map(|s| s.to_string()),
            timestamp: *entry.created_at(),
            session_id: None,
            platform_id: None,
        })
    }
}