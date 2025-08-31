// Permission Service - Table-only permissions after migration completion
// Provides access to user permissions stored in separate user_permissions table

use std::sync::Arc;
use crate::app::ports::repositories::{UserPermissionRepository, RepoError};
use crate::dom::entities::User;
use crate::dom::values::UserId;
use crate::auth::permissions::check_permission_access;

/// Permission service for table-based permissions (post-migration)
pub struct PermissionService {
    user_permission_repo: Arc<dyn UserPermissionRepository>,
}

impl PermissionService {
    pub fn new(user_permission_repo: Arc<dyn UserPermissionRepository>) -> Self {
        Self {
            user_permission_repo,
        }
    }

    /// Get user permissions from separate table
    pub async fn get_user_permissions(&self, user: &User) -> Result<Vec<String>, RepoError> {
        self.user_permission_repo.get_active_permissions(user.id()).await
    }

    /// Check if user has a specific permission
    pub async fn has_permission(&self, user: &User, required_permission: &str) -> Result<bool, RepoError> {
        let permissions = self.get_user_permissions(user).await?;
        Ok(check_permission_access(&permissions, required_permission))
    }

    /// Set user permissions in separate table
    pub async fn set_user_permissions(&self, user_id: &UserId, permissions: Vec<String>) -> Result<(), RepoError> {
        self.user_permission_repo.set_user_permissions(user_id, permissions).await
    }

    /// Grant a specific permission to a user
    pub async fn grant_permission(&self, user_id: &UserId, permission: &str) -> Result<(), RepoError> {
        use crate::dom::entities::UserPermission;
        let user_permission = UserPermission::system_permission(
            user_id.clone(),
            permission.to_string(),
        );
        self.user_permission_repo.grant_permission(&user_permission).await
    }

    /// Revoke a specific permission from a user
    pub async fn revoke_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, RepoError> {
        self.user_permission_repo.revoke_user_permission(user_id, permission).await
    }

    /// Check if user has admin permissions  
    pub async fn is_admin(&self, user: &User) -> Result<bool, RepoError> {
        self.has_permission(user, "admin:*:*").await
    }

    /// Get all user permissions with metadata
    pub async fn get_permissions_with_metadata(&self, user: &User) -> Result<Vec<crate::dom::entities::UserPermission>, RepoError> {
        self.user_permission_repo.get_permissions_with_metadata(user.id()).await
    }
}

/// Factory for creating PermissionService
pub struct PermissionServiceFactory;

impl PermissionServiceFactory {
    /// Create PermissionService (table-only mode)
    pub fn create(
        user_permission_repo: Arc<dyn UserPermissionRepository>,
    ) -> PermissionService {
        PermissionService::new(user_permission_repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::UserPermission;
    use crate::dom::values::{UserId, Email};

    // Mock implementation for testing
    struct MockUserPermissionRepo {
        permissions: std::sync::Mutex<std::collections::HashMap<UserId, Vec<String>>>,
    }

    impl MockUserPermissionRepo {
        fn new() -> Self {
            Self {
                permissions: std::sync::Mutex::new(std::collections::HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserPermissionRepository for MockUserPermissionRepo {
        async fn get_user_permissions(&self, _user_id: &UserId) -> Result<Vec<UserPermission>, RepoError> {
            unimplemented!()
        }
        
        async fn get_permission(&self, _permission_id: &crate::dom::entities::PermissionId) -> Result<Option<UserPermission>, RepoError> {
            unimplemented!()
        }
        
        async fn grant_permission(&self, _permission: &UserPermission) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn revoke_permission(&self, _permission_id: &crate::dom::entities::PermissionId) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn revoke_user_permission(&self, _user_id: &UserId, _permission: &str) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn update_permission(&self, _permission: &UserPermission) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn set_user_permissions(&self, user_id: &UserId, permissions: Vec<String>) -> Result<(), RepoError> {
            let mut perms = self.permissions.lock().unwrap();
            perms.insert(user_id.clone(), permissions);
            Ok(())
        }
        
        async fn has_permission(&self, _user_id: &UserId, _permission: &str) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn get_active_permissions(&self, user_id: &UserId) -> Result<Vec<String>, RepoError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.get(user_id).cloned().unwrap_or_default())
        }
        
        async fn get_permissions_with_metadata(&self, _user_id: &UserId) -> Result<Vec<UserPermission>, RepoError> {
            Ok(vec![])
        }
        
        async fn cleanup_expired_permissions(&self) -> Result<u64, RepoError> {
            Ok(0)
        }
        
        async fn get_permissions_granted_by(&self, _granted_by: &UserId) -> Result<Vec<UserPermission>, RepoError> {
            Ok(vec![])
        }
        
        async fn grant_permissions_batch(&self, _permissions: Vec<UserPermission>) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn find_users_with_permission(&self, _permission: &str) -> Result<Vec<UserId>, RepoError> {
            Ok(vec![])
        }
        
        async fn get_permission_stats(&self) -> Result<crate::app::ports::repositories::PermissionStats, RepoError> {
            Ok(crate::app::ports::repositories::PermissionStats {
                total_permissions: 0,
                active_permissions: 0,
                expired_permissions: 0,
                users_with_permissions: 0,
                most_common_permissions: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_table_only_permissions() {
        let mock_repo = Arc::new(MockUserPermissionRepo::new());
        let service = PermissionServiceFactory::create(mock_repo);

        let user = User::new(
            "test_uid".to_string(),
            Email::new("test@example.com".to_string()).unwrap(),
        );

        // Should return empty permissions when table is empty
        let permissions = service.get_user_permissions(&user).await.unwrap();
        assert!(permissions.is_empty());
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let mock_repo = Arc::new(MockUserPermissionRepo::new());
        let service = PermissionServiceFactory::create(mock_repo);

        let user = User::new(
            "test_uid".to_string(),
            Email::new("test@example.com".to_string()).unwrap(),
        );

        // Test permission checking with empty permissions
        let has_admin = service.has_permission(&user, "admin:*:*").await.unwrap();
        assert!(!has_admin);
    }
}