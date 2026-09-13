//! Database models for the unified permissions system (sqlx-friendly).
//!
//! BIG-BANG: migrated from diesel to plain sqlx structs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Row model for permissions table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionDb {
    pub id: Uuid,
    pub permission_string: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
    pub permission_type: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

/// Insert model for permissions.
#[derive(Debug, Clone, Deserialize)]
pub struct NewPermissionDb {
    pub permission_string: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub is_system: bool,
    pub permission_type: String,
}

/// Update model for permissions.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdatePermissionDb {
    pub is_active: Option<bool>,
    pub description: Option<Option<String>>,
    pub name: Option<Option<String>>,
    pub category: Option<Option<String>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Row model for wallet_direct_permissions table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WalletDirectPermissionDb {
    pub id: Uuid,
    pub wallet_address: String,
    pub permission_id: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
    pub is_active: bool,
}

/// Insert model for wallet_direct_permissions.
#[derive(Debug, Clone, Deserialize)]
pub struct NewWalletDirectPermissionDb {
    pub wallet_address: String,
    pub permission_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by: Option<String>,
    pub grant_reason: Option<String>,
    pub is_active: bool,
}

/// Form data for creating permissions.
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

/// Form data for updating permissions.
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    pub is_active: Option<bool>,
    pub expires_at: Option<Option<DateTime<Utc>>>,
    pub reason: Option<String>,
}

/// Form data for bulk permission assignments.
#[derive(Debug, Deserialize)]
pub struct BulkPermissionRequest {
    pub wallet_addresses: Vec<String>,
    pub permission_string: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

/// Permission statistics result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStats {
    pub total_permissions: i64,
    pub direct_permissions: i64,
    pub group_permissions: i64,
    pub temporary_permissions: i64,
}

/// Platform-wise permission statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPermissionStats {
    pub platform: String,
    pub permission_count: i64,
    pub wallet_count: i64,
}

/// Permission validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionValidationResult {
    pub valid: bool,
    pub permission_string: String,
    pub wallet_address: String,
    pub granted_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub error: Option<String>,
}

/// Permission assignment result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAssignmentResult {
    pub success: bool,
    pub permission_id: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub error: Option<String>,
}

/// Permission search filters.
#[derive(Debug, Deserialize)]
pub struct PermissionSearchFilters {
    pub wallet_address: Option<String>,
    pub platform: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub source_type: Option<String>,
    pub is_active: Option<bool>,
    pub include_expired: Option<bool>,
    pub search_term: Option<String>,
}

/// Permission summary for admin display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSummary {
    pub id: Uuid,
    pub wallet_address: String,
    pub permission_string: String,
    pub platform: String,
    pub resource: String,
    pub action: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub granted_by: Option<String>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub is_expired: bool,
    pub grant_reason: Option<String>,
}

impl PermissionDb {
    pub fn is_currently_active(&self) -> bool {
        self.is_active
    }

    pub fn is_system_permission(&self) -> bool {
        self.is_system
    }
}

impl NewPermissionDb {
    pub fn new(
        permission_string: String,
        description: Option<String>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<Self, String> {
        let parts: Vec<&str> = permission_string.split(':').collect();
        if parts.len() != 3 {
            return Err(
                "Permission string must be in format 'platform:resource:action'".to_string(),
            );
        }

        let platform = parts[0].to_string();
        let resource = parts[1].to_string();
        let action = parts[2].to_string();

        Ok(Self {
            permission_string,
            platform,
            resource,
            action,
            name,
            category,
            description,
            is_system: false,
            permission_type: "manual".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_permission() {
        let result = NewPermissionDb::new(
            "admin:users:manage".to_string(),
            Some("Description".to_string()),
            Some("Name".to_string()),
            Some("Category".to_string()),
        );

        assert!(result.is_ok());
        let perm = result.unwrap();
        assert_eq!(perm.platform, "admin");
        assert_eq!(perm.resource, "users");
        assert_eq!(perm.action, "manage");
        assert_eq!(perm.name, Some("Name".to_string()));
    }
}
