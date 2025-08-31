use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::infra::db::diesel::schema::user_permissions;
use crate::dom::entities::{UserPermission, PermissionId};
use crate::dom::values::UserId;

// ============================================================================
// USER PERMISSION MODELS - SEPARATE PERMISSION STORAGE
// ============================================================================
// These models manage individual permissions stored in separate table
// Format: "platform:resource:action" (e.g., "epsx:rankings:view:25")

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = user_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUserPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_at: Option<DateTime<Utc>>,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = user_permissions)]
pub struct NewDieselUserPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_at: Option<DateTime<Utc>>,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = user_permissions)]
pub struct UpdateDieselUserPermission {
    pub permission: Option<String>,
    pub granted_at: Option<DateTime<Utc>>,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// CONVERSION IMPLEMENTATIONS
// ============================================================================

impl From<&UserPermission> for NewDieselUserPermission {
    fn from(domain: &UserPermission) -> Self {
        Self {
            id: domain.id().as_uuid(),
            user_id: domain.user_id().0,
            permission: domain.permission().to_string(),
            granted_at: Some(domain.granted_at()),
            granted_by: domain.granted_by().map(|id| id.0),
            expires_at: domain.expires_at(),
            is_active: Some(domain.is_active()),
            created_at: Some(domain.created_at()),
            updated_at: Some(domain.updated_at()),
        }
    }
}

impl From<&UserPermission> for DieselUserPermission {
    fn from(domain: &UserPermission) -> Self {
        Self {
            id: domain.id().as_uuid(),
            user_id: domain.user_id().0,
            permission: domain.permission().to_string(),
            granted_at: Some(domain.granted_at()),
            granted_by: domain.granted_by().map(|id| id.0),
            expires_at: domain.expires_at(),
            is_active: Some(domain.is_active()),
            created_at: Some(domain.created_at()),
            updated_at: Some(domain.updated_at()),
        }
    }
}

impl TryFrom<DieselUserPermission> for UserPermission {
    type Error = String;
    
    fn try_from(diesel: DieselUserPermission) -> Result<Self, Self::Error> {
        Ok(UserPermission::reconstruct(
            PermissionId::from_uuid(diesel.id),
            UserId(diesel.user_id),
            diesel.permission,
            diesel.granted_at.unwrap_or_else(Utc::now),
            diesel.granted_by.map(UserId),
            diesel.expires_at,
            diesel.is_active.unwrap_or(true),
            diesel.created_at.unwrap_or_else(Utc::now),
            diesel.updated_at.unwrap_or_else(Utc::now),
        ))
    }
}

// ============================================================================
// HELPER FUNCTIONS FOR BATCH OPERATIONS
// ============================================================================

impl NewDieselUserPermission {
    pub fn from_domain_batch(permissions: &[UserPermission]) -> Vec<Self> {
        permissions.iter().map(Self::from).collect()
    }
    
    pub fn system_permission(user_id: Uuid, permission: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            permission,
            granted_at: Some(now),
            granted_by: None, // System grant
            expires_at: None,  // No expiration
            is_active: Some(true),
            created_at: Some(now),
            updated_at: Some(now),
        }
    }
    
    pub fn temporary_permission(
        user_id: Uuid,
        permission: String,
        granted_by: Uuid,
        expires_at: DateTime<Utc>
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            permission,
            granted_at: Some(now),
            granted_by: Some(granted_by),
            expires_at: Some(expires_at),
            is_active: Some(true),
            created_at: Some(now),
            updated_at: Some(now),
        }
    }
}

// ============================================================================
// QUERY HELPERS
// ============================================================================

impl DieselUserPermission {
    /// Check if permission is currently valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        let is_active = self.is_active.unwrap_or(true);
        if !is_active {
            return false;
        }
        
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }
        
        true
    }
    
    /// Get just the permission strings from a collection
    pub fn extract_permission_strings(permissions: &[Self]) -> Vec<String> {
        permissions.iter()
            .filter(|p| p.is_valid())
            .map(|p| p.permission.clone())
            .collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_permission_creation() {
        let user_id = Uuid::new_v4();
        let permission = NewDieselUserPermission::system_permission(
            user_id,
            "epsx:rankings:view:25".to_string()
        );
        
        assert_eq!(permission.user_id, user_id);
        assert_eq!(permission.permission, "epsx:rankings:view:25");
        assert_eq!(permission.granted_by, None); // System grant
        assert_eq!(permission.expires_at, None); // No expiration
        assert_eq!(permission.is_active, Some(true));
    }
    
    #[test]
    fn test_temporary_permission_creation() {
        let user_id = Uuid::new_v4();
        let granted_by = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        
        let permission = NewDieselUserPermission::temporary_permission(
            user_id,
            "epsx:admin:temp".to_string(),
            granted_by,
            expires_at
        );
        
        assert_eq!(permission.user_id, user_id);
        assert_eq!(permission.granted_by, Some(granted_by));
        assert_eq!(permission.expires_at, Some(expires_at));
        assert_eq!(permission.is_active, Some(true));
    }
    
    #[test]
    fn test_permission_validation() {
        let now = Utc::now();
        
        // Valid active permission
        let valid_permission = DieselUserPermission {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            permission: "epsx:test".to_string(),
            granted_at: Some(now),
            granted_by: None,
            expires_at: None,
            is_active: Some(true),
            created_at: Some(now),
            updated_at: Some(now),
        };
        assert!(valid_permission.is_valid());
        
        // Inactive permission
        let inactive_permission = DieselUserPermission {
            is_active: Some(false),
            ..valid_permission.clone()
        };
        assert!(!inactive_permission.is_valid());
        
        // Expired permission
        let expired_permission = DieselUserPermission {
            expires_at: Some(now - chrono::Duration::hours(1)),
            ..valid_permission
        };
        assert!(!expired_permission.is_valid());
    }
    
    #[test]
    fn test_permission_string_extraction() {
        let permissions = vec![
            DieselUserPermission {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                permission: "epsx:rankings:view:25".to_string(),
                granted_at: Some(Utc::now()),
                granted_by: None,
                expires_at: None,
                is_active: Some(true),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            },
            DieselUserPermission {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                permission: "epsx:admin:temp".to_string(),
                granted_at: Some(Utc::now()),
                granted_by: None,
                expires_at: Some(Utc::now() - chrono::Duration::hours(1)), // Expired
                is_active: Some(true),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            },
        ];
        
        let permission_strings = DieselUserPermission::extract_permission_strings(&permissions);
        
        // Only valid permissions should be included
        assert_eq!(permission_strings.len(), 1);
        assert_eq!(permission_strings[0], "epsx:rankings:view:25");
    }
    
    #[test]
    fn test_domain_conversion() {
        let user_id = UserId(Uuid::new_v4());
        let domain_permission = UserPermission::system_permission(
            user_id.clone(),
            "epsx:test:permission".to_string()
        );
        
        // Convert to Diesel model
        let diesel_permission = NewDieselUserPermission::from(&domain_permission);
        
        assert_eq!(diesel_permission.user_id, user_id.0);
        assert_eq!(diesel_permission.permission, "epsx:test:permission");
        assert_eq!(diesel_permission.granted_by, None); // System permission
        assert_eq!(diesel_permission.expires_at, None);
        assert_eq!(diesel_permission.is_active, Some(true));
    }
}