// Permission Infrastructure Service - Clean Architecture Infrastructure Layer
// Handles database operations for permission management with dual-read support

use std::sync::Arc;
use crate::auth::permissions::{UserClaims, check_permission_access, PermissionError};
use crate::app::ports::repositories::{UserRepository, RepoError};
use crate::dom::services::PermissionService;
use crate::dom::values::UserId;
use crate::dom::entities::User;

/// Infrastructure service for permission operations with database access
/// Bridges the gap between clean architecture and database operations
pub struct PermissionInfrastructureService {
    user_repo: Arc<dyn UserRepository>,
    permission_service: Arc<PermissionService>,
}

impl PermissionInfrastructureService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        permission_service: Arc<PermissionService>,
    ) -> Self {
        Self {
            user_repo,
            permission_service,
        }
    }

    /// Get user by Firebase UID with permissions from appropriate source
    pub async fn get_user_by_firebase_uid(&self, firebase_uid_param: &str) -> Result<Option<User>, RepoError> {
        self.user_repo.find_by_firebase_uid(firebase_uid_param).await
    }

    /// Get user claims with permissions based on migration phase
    pub async fn get_user_claims_from_infrastructure(&self, firebase_uid_param: &str) -> Result<Option<UserClaims>, RepoError> {
        let user = self.get_user_by_firebase_uid(firebase_uid_param).await?;
        
        if let Some(user) = user {
            // Use PermissionService to get permissions based on migration phase
            let user_permissions = self.permission_service.get_user_permissions(&user).await?;
            
            Ok(Some(UserClaims {
                firebase_uid: user.firebase_uid().to_string(),
                email: user.email().to_string(),
                permissions: user_permissions,
                display_name: None, // TODO: Add to User entity
                name: None,         // TODO: Add to User entity
                avatar_url: None,   // TODO: Add to User entity
                is_active: user.is_active(),
                last_login_at: None, // TODO: Add to User entity
            }))
        } else {
            Ok(None)
        }
    }

    /// Check user permission using infrastructure services
    pub async fn check_user_permission_infrastructure(
        &self,
        firebase_uid_param: &str,
        required_permission: &str
    ) -> Result<bool, RepoError> {
        let claims = self.get_user_claims_from_infrastructure(firebase_uid_param).await?;
        
        if let Some(claims) = claims {
            Ok(check_permission_access(&claims.permissions, required_permission))
        } else {
            Ok(false)
        }
    }

    /// Require permission with infrastructure integration
    pub async fn require_permission_infrastructure(
        &self,
        firebase_uid_param: &str,
        required_permission: &str
    ) -> Result<UserClaims, InfrastructurePermissionError> {
        let claims = self.get_user_claims_from_infrastructure(firebase_uid_param).await?
            .ok_or(InfrastructurePermissionError::UserNotFound)?;
        
        if check_permission_access(&claims.permissions, required_permission) {
            Ok(claims)
        } else {
            Err(InfrastructurePermissionError::InsufficientPermissions)
        }
    }

    /// Require any permission with infrastructure integration
    pub async fn require_any_permission_infrastructure(
        &self,
        firebase_uid_param: &str,
        required_permissions: &[String]
    ) -> Result<UserClaims, InfrastructurePermissionError> {
        let claims = self.get_user_claims_from_infrastructure(firebase_uid_param).await?
            .ok_or(InfrastructurePermissionError::UserNotFound)?;
        
        let has_permission = required_permissions.iter()
            .any(|perm| check_permission_access(&claims.permissions, perm));
            
        if has_permission {
            Ok(claims)
        } else {
            Err(InfrastructurePermissionError::InsufficientPermissions)
        }
    }

    /// Update user permissions through infrastructure
    pub async fn update_user_permissions(
        &self,
        user_id: &UserId,
        new_permissions: Vec<String>
    ) -> Result<(), RepoError> {
        // Use PermissionService to handle the update based on migration phase
        self.permission_service.set_user_permissions(user_id, new_permissions).await
    }

    /// Grant permission through infrastructure
    pub async fn grant_user_permission(
        &self,
        user_id: &UserId,
        permission: &str
    ) -> Result<(), RepoError> {
        self.permission_service.grant_permission(user_id, permission).await
    }

    /// Revoke permission through infrastructure
    pub async fn revoke_user_permission(
        &self,
        user_id: &UserId,
        permission: &str
    ) -> Result<bool, RepoError> {
        self.permission_service.revoke_permission(user_id, permission).await
    }

    /// Get current migration phase for debugging (always table-only now)
    pub fn get_migration_phase(&self) -> String {
        "TableOnly (Migration Completed)".to_string()
    }
}

/// Infrastructure-specific permission errors (includes database errors)
#[derive(Debug, thiserror::Error)]
pub enum InfrastructurePermissionError {
    #[error("Access denied: insufficient permissions")]
    InsufficientPermissions,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Database error: {0}")]
    Database(#[from] RepoError),
    
    #[error("Permission error: {0}")]
    Permission(#[from] PermissionError),
}

/// Factory for creating PermissionInfrastructureService
pub struct PermissionInfrastructureServiceFactory;

impl PermissionInfrastructureServiceFactory {
    /// Create infrastructure service with dependencies
    pub fn create(
        user_repo: Arc<dyn UserRepository>,
        permission_service: Arc<PermissionService>,
    ) -> PermissionInfrastructureService {
        PermissionInfrastructureService::new(user_repo, permission_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ports::UserPermissionRepository;
    use crate::dom::services::PermissionService;
    use crate::dom::values::Email;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock implementations for testing
    struct MockUserRepo {
        users: Mutex<HashMap<String, User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            let mut users = HashMap::new();
            let user = User::new(
                "test_uid".to_string(),
                Email::new("test@example.com".to_string()).unwrap(),
            );
            users.insert("test_uid".to_string(), user);
            
            Self {
                users: Mutex::new(users),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepo {
        async fn get(&self, _id: &UserId) -> Result<Option<User>, RepoError> {
            Ok(None)
        }

        async fn save(&self, _user: &User) -> Result<(), RepoError> {
            Ok(())
        }

        async fn delete(&self, _id: &UserId) -> Result<(), RepoError> {
            Ok(())
        }

        async fn find_by_email(&self, _email: &Email) -> Result<Option<User>, RepoError> {
            Ok(None)
        }

        async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(firebase_uid).cloned())
        }

        async fn find_by_package_tier(&self, _package_tier: &str) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
        }

        async fn list(&self, _offset: u32, _limit: u32) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
        }

        async fn count(&self) -> Result<u64, RepoError> {
            Ok(0)
        }

        async fn save_batch(&self, _users: &[User]) -> Result<(), RepoError> {
            Ok(())
        }

        async fn find_all(&self) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
        }

        async fn find_by_id(&self, _id: &UserId) -> Result<User, RepoError> {
            Err(RepoError::NotFound)
        }

        async fn find_users_for_auto_assignment(&self) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
        }

        async fn count_total_users(&self) -> Result<i64, RepoError> {
            Ok(0)
        }

        async fn is_user_active_since(&self, _user_id: &UserId, _since: chrono::DateTime<chrono::Utc>) -> Result<bool, RepoError> {
            Ok(false)
        }

        async fn has_good_payment_history(&self, _user_id: &UserId, _days: i64) -> Result<bool, RepoError> {
            Ok(false)
        }

        async fn health_check(&self) -> Result<(), RepoError> {
            Ok(())
        }

        async fn search_users(
            &self,
            _filters: &crate::app::ports::UserSearchFilters,
            _offset: u32,
            _limit: u32,
            _sort_by: &str,
            _sort_order: &str
        ) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
        }

        async fn count_search_users(&self, _filters: &crate::app::ports::UserSearchFilters) -> Result<u64, RepoError> {
            Ok(0)
        }
    }

    struct MockUserPermissionRepo;

    #[async_trait::async_trait]
    impl UserPermissionRepository for MockUserPermissionRepo {
        async fn get_user_permissions(&self, _user_id: &UserId) -> Result<Vec<crate::dom::entities::UserPermission>, RepoError> {
            Ok(vec![])
        }
        
        async fn get_permission(&self, _permission_id: &crate::dom::entities::PermissionId) -> Result<Option<crate::dom::entities::UserPermission>, RepoError> {
            Ok(None)
        }
        
        async fn grant_permission(&self, _permission: &crate::dom::entities::UserPermission) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn revoke_permission(&self, _permission_id: &crate::dom::entities::PermissionId) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn revoke_user_permission(&self, _user_id: &UserId, _permission: &str) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn update_permission(&self, _permission: &crate::dom::entities::UserPermission) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn set_user_permissions(&self, _user_id: &UserId, _permissions: Vec<String>) -> Result<(), RepoError> {
            Ok(())
        }
        
        async fn has_permission(&self, _user_id: &UserId, _permission: &str) -> Result<bool, RepoError> {
            Ok(true)
        }
        
        async fn get_active_permissions(&self, _user_id: &UserId) -> Result<Vec<String>, RepoError> {
            Ok(vec!["epsx:analytics:view".to_string()])
        }
        
        async fn get_permissions_with_metadata(&self, _user_id: &UserId) -> Result<Vec<crate::dom::entities::UserPermission>, RepoError> {
            Ok(vec![])
        }
        
        async fn cleanup_expired_permissions(&self) -> Result<u64, RepoError> {
            Ok(0)
        }
        
        async fn get_permissions_granted_by(&self, _granted_by: &UserId) -> Result<Vec<crate::dom::entities::UserPermission>, RepoError> {
            Ok(vec![])
        }
        
        async fn grant_permissions_batch(&self, _permissions: Vec<crate::dom::entities::UserPermission>) -> Result<(), RepoError> {
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
    async fn test_infrastructure_service_creation() {
        let user_repo = Arc::new(MockUserRepo::new());
        let permission_repo = Arc::new(MockUserPermissionRepo);
        let permission_service = Arc::new(PermissionService::new(
            permission_repo.clone(),
        ));

        let service = PermissionInfrastructureServiceFactory::create(
            user_repo,
            permission_service,
        );

        assert_eq!(service.get_migration_phase(), "DualRead");
    }

    #[tokio::test]
    async fn test_get_user_claims() {
        let user_repo = Arc::new(MockUserRepo::new());
        let permission_repo = Arc::new(MockUserPermissionRepo);
        let permission_service = Arc::new(PermissionService::new(
            permission_repo.clone(),
        ));

        let service = PermissionInfrastructureServiceFactory::create(
            user_repo,
            permission_service,
        );

        let claims = service.get_user_claims_from_infrastructure("test_uid").await.unwrap();
        assert!(claims.is_some());
        
        let claims = claims.unwrap();
        assert_eq!(claims.firebase_uid, "test_uid");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.permissions.is_empty());
    }
}