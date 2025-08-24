use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub resource: String,
    pub action: String,
    
    // Time-bound attributes
    pub granted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub auto_revoke: bool,
    
    // Assignment metadata
    pub granted_by: Uuid,
    pub reason: Option<String>,
    pub conditions: serde_json::Value,
    
    // Status tracking
    pub status: TemporaryPermissionStatus,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub revocation_reason: Option<String>,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporaryPermissionStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

impl TemporaryPermission {
    pub fn new(
        user_id: Uuid,
        permission: String,
        resource: String,
        action: String,
        expires_at: DateTime<Utc>,
        granted_by: Uuid,
        reason: Option<String>,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            user_id,
            permission,
            resource,
            action,
            granted_at: now,
            expires_at,
            auto_revoke: true,
            granted_by,
            reason,
            conditions: serde_json::json!({}),
            status: TemporaryPermissionStatus::Active,
            revoked_at: None,
            revoked_by: None,
            revocation_reason: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, TemporaryPermissionStatus::Active) && self.expires_at > Utc::now()
    }
    
    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }
    
    pub fn revoke(&mut self, revoked_by: Uuid, reason: Option<String>) {
        self.status = TemporaryPermissionStatus::Revoked;
        self.revoked_at = Some(Utc::now());
        self.revoked_by = Some(revoked_by);
        self.revocation_reason = reason;
        self.updated_at = Utc::now();
    }
    
    pub fn suspend(&mut self) {
        self.status = TemporaryPermissionStatus::Suspended;
        self.updated_at = Utc::now();
    }
    
    pub fn activate(&mut self) {
        if !self.is_expired() {
            self.status = TemporaryPermissionStatus::Active;
            self.updated_at = Utc::now();
        }
    }
    
    pub fn expire(&mut self) {
        self.status = TemporaryPermissionStatus::Expired;
        self.updated_at = Utc::now();
    }
    
    pub fn extend_expiry(&mut self, new_expires_at: DateTime<Utc>) {
        if new_expires_at > self.expires_at {
            self.expires_at = new_expires_at;
            if self.is_expired() && matches!(self.status, TemporaryPermissionStatus::Expired) {
                self.status = TemporaryPermissionStatus::Active;
            }
            self.updated_at = Utc::now();
        }
    }
    
    pub fn set_conditions(&mut self, conditions: serde_json::Value) {
        self.conditions = conditions;
        self.updated_at = Utc::now();
    }
    
    pub fn touch_updated_at(&mut self) {
        self.updated_at = Utc::now();
    }
    
    // Getters
    pub fn id(&self) -> &Uuid { &self.id }
    pub fn user_id(&self) -> &Uuid { &self.user_id }
    pub fn permission(&self) -> &str { &self.permission }
    pub fn resource(&self) -> &str { &self.resource }
    pub fn action(&self) -> &str { &self.action }
    pub fn granted_at(&self) -> &DateTime<Utc> { &self.granted_at }
    pub fn expires_at(&self) -> &DateTime<Utc> { &self.expires_at }
    pub fn auto_revoke(&self) -> bool { self.auto_revoke }
    pub fn granted_by(&self) -> &Uuid { &self.granted_by }
    pub fn reason(&self) -> &Option<String> { &self.reason }
    pub fn conditions(&self) -> &serde_json::Value { &self.conditions }
    pub fn status(&self) -> &TemporaryPermissionStatus { &self.status }
    pub fn revoked_at(&self) -> &Option<DateTime<Utc>> { &self.revoked_at }
    pub fn revoked_by(&self) -> &Option<Uuid> { &self.revoked_by }
    pub fn revocation_reason(&self) -> &Option<String> { &self.revocation_reason }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
    pub fn updated_at(&self) -> &DateTime<Utc> { &self.updated_at }
}