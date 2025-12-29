//! Diesel Models for Unified Permissions System
//!
//! Database models for the new unified permissions table using Diesel ORM.
//! This replaces the complex multi-table permission system with a single,
//! optimized table that supports direct, group-based, and route permissions.

use chrono::{DateTime, Utc};
use diesel::{Queryable, Selectable, Insertable, AsChangeset, Identifiable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Diesel Queryable model for the unified permissions table
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::primary::permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PermissionDb {
    /// Primary key
    pub id: Uuid,
    /// Complete permission string in format platform:resource:action
    pub permission_string: String,
    /// Extracted platform from permission_string
    pub platform: String,
    /// Extracted resource from permission_string
    pub resource: String,
    /// Extracted action from permission_string
    pub action: String,
    /// Human readable name
    pub name: Option<String>,
    /// Permission description
    pub description: Option<String>,
    /// Permission category
    pub category: Option<String>,
    /// Whether this is a system permission (cannot be deleted)
    pub is_system: bool,
    /// Permission type (legacy field)
    pub permission_type: String,
    /// Whether this permission is currently active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Original creator (legacy field)
    pub created_by: Option<String>,
}

/// Diesel Insertable model for creating new permissions
/// Diesel Insertable model for creating new permissions
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schemas::primary::permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPermissionDb {
    /// Complete permission string
    pub permission_string: String,
    /// Extracted platform component
    pub platform: String,
    /// Extracted resource component
    pub resource: String,
    /// Extracted action component
    pub action: String,
    /// Name
    pub name: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Permission description (optional)
    pub description: Option<String>,
    /// System flag
    pub is_system: bool,
    /// Permission type (legacy, defaults to 'manual')
    pub permission_type: String,
}

/// Diesel AsChangeset model for updating existing permissions
/// Diesel AsChangeset model for updating existing permissions
#[derive(Debug, Clone, AsChangeset, Default)]
#[diesel(table_name = crate::schemas::primary::permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdatePermissionDb {
    /// Update permission status
    pub is_active: Option<bool>,
    /// Update description
    pub description: Option<Option<String>>,
    /// Update name
    pub name: Option<Option<String>>,
    /// Update category
    pub category: Option<Option<String>>,
    /// Force update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

// Temporarily commented out due to Diesel schema generation issues
/*
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schemas::primary::wallet_permissions_view)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletPermissionsViewDb {
    /// Wallet address (primary key)
    pub wallet_address: String,
    /// Whether the wallet is active
    pub wallet_active: bool,
    /// Wallet metadata JSON
    pub wallet_metadata: serde_json::Value,
    /// Wallet creation timestamp
    pub wallet_created: DateTime<Utc>,
    /// Wallet last update timestamp
    pub wallet_updated: DateTime<Utc>,
    /// Last authentication timestamp
    pub last_auth_at: Option<DateTime<Utc>>,
    /// All permissions for this wallet as JSON array
    #[diesel(sql_type = diesel::sql_types::Json)]
    pub permissions: Option<serde_json::Value>,
    /// Total number of active permissions
    pub total_permissions: i64,
    /// Number of direct permissions
    pub direct_permissions: i64,
    /// Number of group-based permissions
    pub group_permissions: i64,
    /// Number of temporary permissions (with expiry)
    pub temporary_permissions: i64,
    /// Activity status: 'active', 'new', 'inactive'
    pub activity_status: String,
    /// When this view was last refreshed
    pub view_refreshed_at: DateTime<Utc>,
}
*/

/// Form data for creating permissions from API requests
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    /// Wallet address to grant permission to
    pub wallet_address: String,
    /// Permission string in format platform:resource:action
    pub permission_string: String,
    /// Source type: 'direct', 'group', 'route'
    pub source_type: String,
    /// Source ID for group-based permissions
    pub source_id: Option<Uuid>,
    /// Optional expiry time for temporary permissions
    pub expires_at: Option<DateTime<Utc>>,
    /// Reason for granting this permission
    pub reason: Option<String>,
}

/// Form data for updating permissions from API requests
#[derive(Debug, Deserialize)]
pub struct UpdatePermissionRequest {
    /// Update permission status
    pub is_active: Option<bool>,
    /// Update expiry time (NULL to make permanent)
    pub expires_at: Option<Option<DateTime<Utc>>>,
    /// Update reason
    pub reason: Option<String>,
}

/// Form data for bulk permission assignments
#[derive(Debug, Deserialize)]
pub struct BulkPermissionRequest {
    /// Wallet addresses to assign permissions to
    pub wallet_addresses: Vec<String>,
    /// Permission string to assign
    pub permission_string: String,
    /// Source type
    pub source_type: String,
    /// Source ID for group-based
    pub source_id: Option<Uuid>,
    /// Optional expiry for all assignments
    pub expires_at: Option<DateTime<Utc>>,
    /// Reason for bulk assignment
    pub reason: Option<String>,
}

/// Permission statistics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStats {
    /// Total active permissions for wallet
    pub total_permissions: i64,
    /// Number of direct permissions
    pub direct_permissions: i64,
    /// Number of group-based permissions
    pub group_permissions: i64,
    /// Number of temporary (expiring) permissions
    pub temporary_permissions: i64,
}

/// Platform-wise permission statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPermissionStats {
    /// Platform name
    pub platform: String,
    /// Total permissions for this platform
    pub permission_count: i64,
    /// Number of unique wallets with this platform's permissions
    pub wallet_count: i64,
}

/// Permission validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionValidationResult {
    /// Whether the validation passed
    pub valid: bool,
    /// The permission string that was validated
    pub permission_string: String,
    /// Wallet address that was checked
    pub wallet_address: String,
    /// When the permission was granted (if valid)
    pub granted_at: Option<DateTime<Utc>>,
    /// When the permission expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Source of the permission
    pub source_type: Option<String>,
    /// Validation error message (if invalid)
    pub error: Option<String>,
}

/// Permission assignment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAssignmentResult {
    /// Whether the assignment succeeded
    pub success: bool,
    /// ID of the created permission
    pub permission_id: Option<Uuid>,
    /// When the permission was granted
    pub granted_at: DateTime<Utc>,
    /// Assignment error (if failed)
    pub error: Option<String>,
}

/// Permission search filters
#[derive(Debug, Deserialize)]
pub struct PermissionSearchFilters {
    /// Filter by wallet address
    pub wallet_address: Option<String>,
    /// Filter by platform
    pub platform: Option<String>,
    /// Filter by resource
    pub resource: Option<String>,
    /// Filter by action
    pub action: Option<String>,
    /// Filter by source type
    pub source_type: Option<String>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Filter by expiry status
    pub include_expired: Option<bool>,
    /// Search in permission string
    pub search_term: Option<String>,
}

/// Permission summary for admin display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSummary {
    /// Permission ID
    pub id: Uuid,
    /// Wallet address
    pub wallet_address: String,
    /// Permission string
    pub permission_string: String,
    /// Platform component
    pub platform: String,
    /// Resource component
    pub resource: String,
    /// Action component
    pub action: String,
    /// Source type
    pub source_type: String,
    /// Source ID
    pub source_id: Option<Uuid>,
    /// Granted by
    pub granted_by: Option<String>,
    /// Granted at
    pub granted_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
    /// Is active
    pub is_active: bool,
    /// Is expired
    pub is_expired: bool,
    /// Grant reason
    pub grant_reason: Option<String>,
}

/// Helper functions for permission management
/// Helper functions for permission management
impl PermissionDb {
    /// Check if this permission is currently active
    pub fn is_currently_active(&self) -> bool {
        self.is_active
    }

    /// Check if this is a system permission
    pub fn is_system_permission(&self) -> bool {
        self.is_system
    }
}

    /// Create a simplified permission definition
impl NewPermissionDb {
    pub fn new(
        permission_string: String,
        description: Option<String>,
        name: Option<String>,
        category: Option<String>,
    ) -> Result<Self, String> {
        // Parse permission string into components
        let parts: Vec<&str> = permission_string.split(':').collect();
        if parts.len() != 3 {
            return Err("Permission string must be in format 'platform:resource:action'".to_string());
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

// Temporarily commented out due to Diesel schema generation issues
/*
/// Helper functions for wallet permissions view
impl WalletPermissionsViewDb {
    /// Get permissions as a structured list
    pub fn get_permissions_list(&self) -> Vec<PermissionSummary> {
        if let Some(permissions_json) = &self.permissions {
            if let Some(permissions_array) = permissions_json.as_array() {
                permissions_array
                    .iter()
                    .filter_map(|perm| {
                        serde_json::from_value::<PermissionSummary>(perm.clone()).ok()
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Check if wallet is considered active
    pub fn is_active_wallet(&self) -> bool {
        self.wallet_active && self.activity_status == "active"
    }

    /// Check if wallet has any permissions
    pub fn has_permissions(&self) -> bool {
        self.total_permissions > 0
    }

    /// Get permission breakdown percentages
    pub fn get_permission_breakdown(&self) -> (f64, f64, f64) {
        if self.total_permissions == 0 {
            return (0.0, 0.0, 0.0);
        }

        let direct_pct = (self.direct_permissions as f64 / self.total_permissions as f64) * 100.0;
        let group_pct = (self.group_permissions as f64 / self.total_permissions as f64) * 100.0;
        let temp_pct = (self.temporary_permissions as f64 / self.total_permissions as f64) * 100.0;

        (direct_pct, group_pct, temp_pct)
    }
}
*/

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