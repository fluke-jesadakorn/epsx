// ============================================================================
// TEMPORARY PERMISSION ENTITY STUB - REPLACING COMPLEX TEMP PERMISSIONS
// ============================================================================
// Simple stub for removed temporary permission system

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::dom::values::UserId;

// ============================================================================
// TEMPORARY PERMISSION TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemporaryPermissionStatus {
    Active,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryPermission {
    pub id: Uuid,
    pub user_id: UserId,
    pub permission: String,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub status: TemporaryPermissionStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: UserId,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub reason: String,
}

impl TemporaryPermission {
    pub fn new(
        user_id: UserId,
        permission: String,
        granted_by: UserId,
        expires_at: Option<DateTime<Utc>>,
        reason: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            permission,
            resource: None,
            action: None,
            status: TemporaryPermissionStatus::Active,
            expires_at,
            granted_by,
            granted_at: Utc::now(),
            revoked_at: None,
            reason,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, TemporaryPermissionStatus::Active) &&
            self.expires_at.map_or(true, |exp| exp > Utc::now())
    }

    pub fn revoke(&mut self) {
        self.status = TemporaryPermissionStatus::Revoked;
        self.revoked_at = Some(Utc::now());
    }

    pub fn expire(&mut self) {
        self.status = TemporaryPermissionStatus::Expired;
    }
}

impl Default for TemporaryPermission {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: UserId::generate(),
            permission: "default".to_string(),
            resource: None,
            action: None,
            status: TemporaryPermissionStatus::Active,
            expires_at: None,
            granted_by: UserId::generate(),
            granted_at: Utc::now(),
            revoked_at: None,
            reason: "default".to_string(),
        }
    }
}