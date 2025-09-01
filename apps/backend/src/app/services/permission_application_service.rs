// Permission Application Service - Clean Architecture Application Layer
// Orchestrates permission operations across domain and infrastructure layers

use std::sync::Arc;
use crate::auth::permissions::{UserClaims, PermissionError};
use crate::dom::services::PermissionService;
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email};
use crate::app::ports::repositories::{UserRepository, RepoError};
use crate::infra::services::permission_infrastructure::{
    PermissionInfrastructureService, InfrastructurePermissionError
};

/// Application service for permission management operations
/// Orchestrates between domain services and infrastructure services
pub struct PermissionApplicationService {
    permission_service: Arc<PermissionService>,
    infrastructure_service: Arc<PermissionInfrastructureService>,
    user_repo: Arc<dyn UserRepository>,
}

impl PermissionApplicationService {
    pub fn new(
        permission_service: Arc<PermissionService>,
        infrastructure_service: Arc<PermissionInfrastructureService>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            permission_service,
            infrastructure_service,
            user_repo,
        }
    }

    // ============================================================================
    // USER PERMISSION QUERIES
    // ============================================================================

    /// Get user claims with permissions (main entry point for authentication)
    pub async fn get_user_claims(&self, firebase_uid: &str) -> Result<Option<UserClaims>, ApplicationPermissionError> {
        self.infrastructure_service
            .get_user_claims_from_infrastructure(firebase_uid)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)
    }

    /// Check if user has specific permission
    pub async fn check_user_permission(
        &self,
        firebase_uid: &str,
        required_permission: &str
    ) -> Result<bool, ApplicationPermissionError> {
        self.infrastructure_service
            .check_user_permission_infrastructure(firebase_uid, required_permission)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)
    }

    /// Get user permissions based on migration phase
    pub async fn get_user_permissions(&self, firebase_uid: &str) -> Result<Vec<String>, ApplicationPermissionError> {
        let user = self.infrastructure_service
            .get_user_by_firebase_uid(firebase_uid)
            .await
            .map_err(|e| ApplicationPermissionError::Infrastructure(e))?
            .ok_or(ApplicationPermissionError::UserNotFound)?;

        self.permission_service
            .get_user_permissions(&user)
            .await
            .map_err(|e| ApplicationPermissionError::Infrastructure(e))
    }

    // ============================================================================
    // AUTHENTICATION OPERATIONS
    // ============================================================================

    /// Require permission (throws error if insufficient)
    pub async fn require_permission(
        &self,
        firebase_uid: &str,
        required_permission: &str
    ) -> Result<UserClaims, ApplicationPermissionError> {
        self.infrastructure_service
            .require_permission_infrastructure(firebase_uid, required_permission)
            .await
            .map_err(ApplicationPermissionError::InfrastructurePermission)
    }

    /// Require any of the specified permissions
    pub async fn require_any_permission(
        &self,
        firebase_uid: &str,
        required_permissions: &[String]
    ) -> Result<UserClaims, ApplicationPermissionError> {
        self.infrastructure_service
            .require_any_permission_infrastructure(firebase_uid, required_permissions)
            .await
            .map_err(ApplicationPermissionError::InfrastructurePermission)
    }

    // ============================================================================
    // PERMISSION MANAGEMENT OPERATIONS
    // ============================================================================

    /// Grant permission to user
    pub async fn grant_permission_to_user(
        &self,
        user_id: &UserId,
        permission: &str,
        _granted_by: Option<&UserId>
    ) -> Result<(), ApplicationPermissionError> {
        // Business rule: Validate permission format
        if !self.is_valid_permission_format(permission) {
            return Err(ApplicationPermissionError::InvalidPermissionFormat(
                permission.to_string()
            ));
        }

        // Use domain service to grant permission
        self.permission_service
            .grant_permission(user_id, permission)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)
    }

    /// Revoke permission from user
    pub async fn revoke_permission_from_user(
        &self,
        user_id: &UserId,
        permission: &str
    ) -> Result<bool, ApplicationPermissionError> {
        self.permission_service
            .revoke_permission(user_id, permission)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)
    }

    /// Set complete permission set for user
    pub async fn set_user_permissions(
        &self,
        user_id: &UserId,
        permissions: Vec<String>
    ) -> Result<(), ApplicationPermissionError> {
        // Business rule: Validate all permissions
        for permission in &permissions {
            if !self.is_valid_permission_format(permission) {
                return Err(ApplicationPermissionError::InvalidPermissionFormat(
                    permission.to_string()
                ));
            }
        }

        // Use domain service to set permissions
        self.permission_service
            .set_user_permissions(user_id, permissions)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)
    }

    // ============================================================================
    // USER MANAGEMENT OPERATIONS
    // ============================================================================

    /// Create new user with default permissions
    pub async fn create_user_with_permissions(
        &self,
        firebase_uid: String,
        email: Email,
        initial_permissions: Option<Vec<String>>
    ) -> Result<User, ApplicationPermissionError> {
        // Create user entity
        let user = User::new(firebase_uid, email);

        // Save user through repository first
        self.user_repo
            .save(&user)
            .await
            .map_err(ApplicationPermissionError::Infrastructure)?;

        // Set initial permissions in separate table
        if let Some(permissions) = initial_permissions {
            for permission in &permissions {
                if !self.is_valid_permission_format(permission) {
                    return Err(ApplicationPermissionError::InvalidPermissionFormat(permission.clone()));
                }
            }
            
            // Set permissions in separate table
            self.permission_service
                .set_user_permissions(user.id(), permissions)
                .await
                .map_err(ApplicationPermissionError::Infrastructure)?;
        }

        Ok(user)
    }

    /// Update user permissions (handles migration phases)
    pub async fn update_user_permissions(
        &self,
        firebase_uid: &str,
        new_permissions: Vec<String>
    ) -> Result<(), ApplicationPermissionError> {
        let user = self.infrastructure_service
            .get_user_by_firebase_uid(firebase_uid)
            .await
            .map_err(|e| ApplicationPermissionError::Infrastructure(e))?
            .ok_or(ApplicationPermissionError::UserNotFound)?;

        self.set_user_permissions(user.id(), new_permissions).await
    }

    // ============================================================================
    // MIGRATION OPERATIONS - COMPLETED (Migration to table-only mode is complete)
    // ============================================================================

    /// Get current migration phase (always table-only now)
    pub fn get_migration_phase(&self) -> String {
        "TableOnly (Migration Completed)".to_string()
    }

    /// Check if dual-read is active (always false now)
    pub fn is_dual_read_active(&self) -> bool {
        false
    }

    /// Check if table-only mode is active (always true now)
    pub fn is_table_only_active(&self) -> bool {
        true
    }

    // ============================================================================
    // VALIDATION HELPERS
    // ============================================================================

    /// Validate permission format (domain business rule)
    /// Supports both standard format (platform:resource:action) and embedded timestamp format (platform:resource:action:timestamp)
    fn is_valid_permission_format(&self, permission: &str) -> bool {
        let parts: Vec<&str> = permission.split(':').collect();
        
        // Standard format: platform:resource:action (3 parts)
        if parts.len() == 3 {
            return !parts[0].is_empty() && !parts[1].is_empty() && !parts[2].is_empty();
        }
        
        // Embedded timestamp format: platform:resource:action:timestamp (4 parts)
        if parts.len() == 4 {
            let timestamp_valid = parts[3].parse::<i64>().is_ok();
            return !parts[0].is_empty() && 
                   !parts[1].is_empty() && 
                   !parts[2].is_empty() && 
                   timestamp_valid;
        }
        
        false
    }

    // ============================================================================
    // ANALYTICS AND REPORTING
    // ============================================================================

    /// Get permission statistics (for admin dashboard)
    pub async fn get_permission_statistics(&self) -> Result<PermissionStatistics, ApplicationPermissionError> {
        // This would be implemented by querying the permission repository
        // For now, return basic stats
        Ok(PermissionStatistics {
            total_users_with_permissions: 0,
            total_permissions_granted: 0,
            most_common_permissions: vec![],
            migration_phase: self.get_migration_phase(),
        })
    }
}

// ============================================================================
// APPLICATION LAYER TYPES
// ============================================================================

/// Application-specific permission error
#[derive(Debug, thiserror::Error)]
pub enum ApplicationPermissionError {
    #[error("User not found")]
    UserNotFound,

    #[error("Access denied: insufficient permissions")]
    InsufficientPermissions,

    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(RepoError),

    #[error("Domain error: {0}")]
    Domain(#[from] PermissionError),

    #[error("Infrastructure permission error: {0}")]
    InfrastructurePermission(#[from] InfrastructurePermissionError),
}

/// Permission statistics for reporting
#[derive(Debug, Clone)]
pub struct PermissionStatistics {
    pub total_users_with_permissions: u64,
    pub total_permissions_granted: u64,
    pub most_common_permissions: Vec<String>,
    pub migration_phase: String,
}

/// Factory for creating PermissionApplicationService
pub struct PermissionApplicationServiceFactory;

impl PermissionApplicationServiceFactory {
    /// Create application service with all dependencies
    pub fn create(
        permission_service: Arc<PermissionService>,
        infrastructure_service: Arc<PermissionInfrastructureService>,
        user_repo: Arc<dyn UserRepository>,
    ) -> PermissionApplicationService {
        PermissionApplicationService::new(
            permission_service,
            infrastructure_service,
            user_repo,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::services::PermissionService;
    use crate::dom::values::Email;
    use crate::infra::services::permission_infrastructure::PermissionInfrastructureServiceFactory;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock implementations for testing
    struct MockUserRepo {
        users: Mutex<HashMap<String, User>>,
    }

    impl MockUserRepo {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepo {
        async fn get_user_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(firebase_uid).cloned())
        }

        async fn create_user(&self, user: &User) -> Result<(), RepoError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.firebase_uid().to_string(), user.clone());
            Ok(())
        }

        async fn update_user(&self, user: &User) -> Result<(), RepoError> {
            let mut users = self.users.lock().unwrap();
            users.insert(user.firebase_uid().to_string(), user.clone());
            Ok(())
        }

        async fn delete_user(&self, _user_id: &UserId) -> Result<bool, RepoError> {
            Ok(true)
        }

        async fn get_user_by_id(&self, _user_id: &UserId) -> Result<Option<User>, RepoError> {
            Ok(None)
        }

        async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>, RepoError> {
            Ok(None)
        }

        async fn list_users(&self, _limit: Option<i64>, _offset: Option<i64>) -> Result<Vec<User>, RepoError> {
            Ok(vec![])
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
    async fn test_permission_format_validation() {
        let user_repo = Arc::new(MockUserRepo::new());
        let permission_repo = Arc::new(MockUserPermissionRepo);
        let permission_service = Arc::new(PermissionService::new(
            permission_repo.clone(),
        ));
        let infrastructure_service = Arc::new(PermissionInfrastructureServiceFactory::create(
            user_repo.clone(),
            permission_service.clone(),
        ));

        let app_service = PermissionApplicationServiceFactory::create(
            permission_service,
            infrastructure_service,
            user_repo,
        );

        // Test valid permission format
        assert!(app_service.is_valid_permission_format("epsx:analytics:view"));
        
        // Test invalid formats
        assert!(!app_service.is_valid_permission_format("invalid"));
        assert!(!app_service.is_valid_permission_format("epsx:analytics"));
        assert!(!app_service.is_valid_permission_format("::"));
    }

    #[tokio::test]
    async fn test_create_user_with_permissions() {
        let user_repo = Arc::new(MockUserRepo::new());
        let permission_repo = Arc::new(MockUserPermissionRepo);
        let permission_service = Arc::new(PermissionService::new(
            permission_repo.clone(),
        ));
        let infrastructure_service = Arc::new(PermissionInfrastructureServiceFactory::create(
            user_repo.clone(),
            permission_service.clone(),
        ));

        let app_service = PermissionApplicationServiceFactory::create(
            permission_service,
            infrastructure_service,
            user_repo,
        );

        let email = Email::new("test@example.com".to_string()).unwrap();
        let permissions = vec!["epsx:analytics:view".to_string(), "epsx:profile:manage".to_string()];

        let user = app_service.create_user_with_permissions(
            "test_uid".to_string(),
            email,
            Some(permissions.clone())
        ).await.unwrap();

        assert_eq!(user.firebase_uid(), "test_uid");
        
        // Fetch permissions from service since User no longer has permissions() method
        let user_permissions = service.get_user_permissions(user.firebase_uid()).await.unwrap();
        assert_eq!(user_permissions.len(), permissions.len());
        assert!(user_permissions.contains(&"epsx:analytics:view".to_string()));
        assert!(user_permissions.contains(&"epsx:profile:manage".to_string()));
    }
}