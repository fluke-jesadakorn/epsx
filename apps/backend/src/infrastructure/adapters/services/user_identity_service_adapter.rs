// User Identity Service Adapter
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use std::sync::Arc;

use crate::domain::authentication::{
    UserIdentityServicePort, AuthenticatedUserId
};
use crate::application::ports::repositories::UserPermissionRepository;
use crate::domain::shared_kernel::entities::user::User as LegacyUser;
use crate::domain::shared_kernel::value_objects::UserId as LegacyUserId;
use crate::domain::user_management::value_objects::Email;

/// User identity service adapter
pub struct UserIdentityServiceAdapter {
    /// User repository through domain port
    user_repository: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
    
    /// User permissions repository (using LegacyPermissionRepositoryError for std::error::Error compliance)
    permission_repository: Arc<dyn UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>>,
}

impl UserIdentityServiceAdapter {
    pub fn new(
        user_repository: Arc<dyn crate::domain::user_management::UserRepositoryPort>,
        permission_repository: Arc<dyn UserPermissionRepository<Error = crate::infrastructure::adapters::repositories::user_permission_repository_adapter::LegacyPermissionRepositoryError>>,
    ) -> Self {
        Self {
            user_repository,
            permission_repository,
        }
    }
    
    /// Convert legacy user to identity information
    fn map_legacy_user_to_identity(&self, legacy_user: &LegacyUser) -> UserIdentityInfo {
        UserIdentityInfo {
            user_id: legacy_user.id().to_string(),
            firebase_uid: Some(legacy_user.firebase_uid().to_string()),
            email: legacy_user.email().to_string(),
            is_verified: legacy_user.is_active(), // Use is_active as proxy for email verification
            is_active: legacy_user.is_active(),
            created_at: chrono::Utc::now(), // created_at method not available, use current time
            last_login: None, // Would come from session data
            subscription_tier: Some("free".to_string()), // Default subscription tier since field not available
        }
    }
}

#[async_trait]
impl UserIdentityServicePort for UserIdentityServiceAdapter {
    async fn get_user_identity(&self, user_id: &AuthenticatedUserId) -> Result<crate::domain::authentication::repositories::UserProfile, String> {
        info!(user_id = %user_id, "Getting user identity information");
        
        // Convert to legacy UserId for repository
        let legacyuser_id = LegacyUserId::from_string(user_id.user_id().to_string())
            .map_err(|e| format!("Invalid user ID format: {}", e))?;
        
        match crate::domain::user_management::UserRepositoryPort::find_by_id(&*self.user_repository, &legacyuser_id).await {
            Ok(Some(legacy_user)) => {
                let identity_info = self.map_legacy_user_to_identity(&legacy_user);
                info!(
                    user_id = %user_id,
                    email = %identity_info.email,
                    "User identity retrieved successfully"
                );
                
                // Convert to UserProfile
                Ok(crate::domain::authentication::repositories::UserProfile {
                    user_id: identity_info.user_id.to_string(),
                    email: identity_info.email,
                    display_name: None, // display_name field not available on UserIdentityInfo
                    avatar_url: None, // avatar_url field not available on UserIdentityInfo
                    created_at: identity_info.created_at,
                    last_login: None,
                    permissions: vec![], // Default empty permissions
                    is_active: identity_info.is_active, // Use actual is_active status
                })
            },
            Ok(None) => {
                warn!(user_id = %user_id, "User not found in identity service");
                Err("User not found".to_string())
            },
            Err(e) => {
                error!(user_id = %user_id, error = %e, "Failed to retrieve user identity");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn get_user_permissions(&self, user_id: &AuthenticatedUserId) -> Result<Vec<String>, String> {
        info!(user_id = %user_id, "Getting user permissions");
        
        // Convert AuthenticatedUserId to domain UserId
        let domainuser_id = user_id.user_id();
        
        match self.permission_repository.get_user_permissions(domainuser_id).await {
            Ok(permissions) => {
                // Permissions are already strings from the repository
                let permission_strings = permissions;
                
                info!(
                    user_id = %user_id,
                    permission_count = permission_strings.len(),
                    "User permissions retrieved"
                );
                
                Ok(permission_strings)
            },
            Err(e) => {
                error!(user_id = %user_id, "Failed to retrieve user permissions: {}", e);
                Err(format!("Permission repository error: {}", e))
            }
        }
    }
    
    async fn validate_user_exists(&self, user_id: &AuthenticatedUserId) -> Result<bool, String> {
        info!(user_id = %user_id, "Validating user exists");
        
        let legacyuser_id = LegacyUserId::from_string(user_id.user_id().to_string())
            .map_err(|e| format!("Invalid user ID format: {}", e))?;
        
        match crate::domain::user_management::UserRepositoryPort::find_by_id(&*self.user_repository, &legacyuser_id).await {
            Ok(Some(_)) => {
                info!(user_id = %user_id, "User exists validation successful");
                Ok(true)
            },
            Ok(None) => {
                info!(user_id = %user_id, "User does not exist");
                Ok(false)
            },
            Err(e) => {
                error!(user_id = %user_id, error = %e, "User existence validation failed");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn get_user_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<crate::domain::authentication::repositories::UserProfile>, String> {
        info!(firebase_uid = firebase_uid, "Getting user by Firebase UID");
        
        // Firebase UID lookup not available in DDD repository, return error
        Err("Firebase UID lookup not implemented in DDD user repository".to_string())
    }
    
    async fn get_user_by_email(&self, email: &str) -> Result<Option<crate::domain::authentication::repositories::UserProfile>, String> {
        info!(email = email, "Getting user by email");
        
        // Convert to Email value object for repository
        let domain_email = Email::new(email.to_string())
            .map_err(|e| format!("Invalid email format: {}", e))?;
        
        match self.user_repository.find_by_email(&domain_email).await {
            Ok(Some(legacy_user)) => {
                let identity_info = self.map_legacy_user_to_identity(&legacy_user);
                info!(
                    email = email,
                    user_id = %identity_info.user_id,
                    "User found by email"
                );
                
                // Convert to UserProfile
                Ok(Some(crate::domain::authentication::repositories::UserProfile {
                    user_id: identity_info.user_id.to_string(),
                    email: identity_info.email,
                    display_name: None, // display_name field not available on UserIdentityInfo
                    avatar_url: None, // avatar_url field not available on UserIdentityInfo
                    created_at: identity_info.created_at,
                    last_login: None,
                    permissions: vec![], // Default empty permissions
                    is_active: identity_info.is_active, // Use actual is_active status
                }))
            },
            Ok(None) => {
                info!(email = email, "No user found for email");
                Ok(None)
            },
            Err(e) => {
                error!(email = email, error = %e, "Failed to find user by email");
                Err(format!("Repository error: {}", e))
            }
        }
    }
    
    async fn update_last_login(&self, user_id: &AuthenticatedUserId, timestamp: DateTime<Utc>) -> Result<(), String> {
        info!(user_id = %user_id, timestamp = %timestamp, "Updating user last login");
        
        let _legacyuser_id = LegacyUserId::from_string(user_id.user_id().to_string())
            .map_err(|e| format!("Invalid user ID format: {}", e))?;
        
        // TODO: Implement last login update when legacy repository supports it
        info!(user_id = %user_id, timestamp = %timestamp, "Last login update not implemented for legacy repository");
        Ok(())
    }
    
    async fn validate_user_access(&self, user_id: &AuthenticatedUserId, required_permission: &str) -> Result<bool, String> {
        info!(
            user_id = %user_id,
            required_permission = required_permission,
            "Validating user access"
        );
        
        // Get user permissions
        let permissions = self.get_user_permissions(user_id).await?;
        
        // Check if user has the required permission
        let has_permission = permissions.iter().any(|p| {
            // Support structured permissions (platform:resource:action)
            if required_permission.contains(':') && p.contains(':') {
                // Exact match or wildcard match
                p == required_permission || 
                p.ends_with(":*") && required_permission.starts_with(&p[..p.len()-1])
            } else {
                // Simple string match
                p == required_permission
            }
        });
        
        // Check for admin permissions (admin:*:* grants all access)
        let is_admin = permissions.iter().any(|p| p.starts_with("admin:") && p.ends_with(":*"));
        
        let has_access = has_permission || is_admin;
        
        if has_access {
            info!(
                user_id = %user_id,
                required_permission = required_permission,
                "User access validation successful"
            );
        } else {
            warn!(
                user_id = %user_id,
                required_permission = required_permission,
                "User access validation failed - insufficient permissions"
            );
        }
        
        Ok(has_access)
    }
    
    async fn get_user_subscription_info(&self, user_id: &AuthenticatedUserId) -> Result<crate::domain::authentication::repositories::UserSubscription, String> {
        info!(user_id = %user_id, "Getting user subscription information");
        
        let _identity_info = self.get_user_identity(user_id).await?;
        
        let subscription_type = crate::domain::authentication::repositories::SubscriptionType::Free;
        
        // Convert SubscriptionType to local SubscriptionTier for compatibility
        let subscription_tier = match subscription_type {
            crate::domain::authentication::repositories::SubscriptionType::Free => SubscriptionTier::Free,
            crate::domain::authentication::repositories::SubscriptionType::Premium => SubscriptionTier::Premium,
            crate::domain::authentication::repositories::SubscriptionType::Enterprise => SubscriptionTier::Enterprise,
        };
        
        let subscription_info = crate::domain::authentication::repositories::UserSubscription {
            user_id: user_id.to_string(),
            subscription_type: subscription_type.clone(),
            is_active: true,
            expires_at: None,
            features: self.get_tier_features(&subscription_tier),
        };
        
        info!(
            user_id = %user_id,
            tier = ?subscription_type,
            "User subscription information retrieved"
        );
        
        Ok(subscription_info)
    }
    
    async fn verify_user_identity(&self, token: &str) -> Result<AuthenticatedUserId, String> {
        info!("Verifying user identity from token");
        
        // For now, return a placeholder implementation
        // In a real implementation, this would:
        // 1. Parse and validate the JWT token
        // 2. Extract user ID from claims
        // 3. Verify token signature
        // 4. Check token expiration
        
        // Placeholder: Extract user ID from token (assuming it's a simple format)
        let user_id_str = token.split('.').next().unwrap_or("1");
        
        // Create shared kernel UserId and convert to AuthenticatedUserId
        use crate::domain::shared_kernel::value_objects::UserId as SharedUserId;
        let shareduser_id = SharedUserId::from_string(user_id_str.to_string())
            .map_err(|e| format!("Invalid user ID in token: {}", e))?;
        
        Ok(AuthenticatedUserId::from_verified_user(shareduser_id))
    }
    
    async fn has_permission(&self, user_id: &AuthenticatedUserId, permission: &str) -> Result<bool, String> {
        info!(
            user_id = %user_id,
            permission = permission,
            "Checking if user has permission"
        );
        
        let permissions = self.get_user_permissions(user_id).await?;
        let has_perm = permissions.iter().any(|p| {
            p == permission || 
            (p.ends_with(":*") && permission.starts_with(&p[..p.len()-1]))
        });
        
        Ok(has_perm)
    }
    
    async fn get_user_profile(&self, user_id: &AuthenticatedUserId) -> Result<crate::domain::authentication::repositories::UserProfile, String> {
        info!(user_id = %user_id, "Getting user profile");
        
        let identity_info = self.get_user_identity(user_id).await?;
        
        Ok(crate::domain::authentication::repositories::UserProfile {
            user_id: identity_info.user_id.to_string(),
            email: identity_info.email,
            display_name: None, // display_name field not available on UserIdentityInfo
            avatar_url: None, // avatar_url field not available on UserIdentityInfo
            created_at: identity_info.created_at,
            last_login: None, // Would come from session service
            permissions: vec![], // Would come from permissions service
            is_active: identity_info.is_active, // Use actual is_active status
        })
    }
}

impl UserIdentityServiceAdapter {
    /// Get features available for subscription tier
    fn get_tier_features(&self, tier: &SubscriptionTier) -> Vec<String> {
        match tier {
            SubscriptionTier::Free => vec![
                "basic_analytics".to_string(),
                "limited_api_calls".to_string(),
            ],
            SubscriptionTier::Premium => vec![
                "basic_analytics".to_string(),
                "advanced_analytics".to_string(),
                "increased_api_calls".to_string(),
                "real_time_data".to_string(),
            ],
            SubscriptionTier::Enterprise => vec![
                "basic_analytics".to_string(),
                "advanced_analytics".to_string(),
                "unlimited_api_calls".to_string(),
                "real_time_data".to_string(),
                "custom_integrations".to_string(),
                "priority_support".to_string(),
            ],
        }
    }
    
    /// Get limits for subscription tier
    fn get_tier_limits(&self, tier: &SubscriptionTier) -> UserLimits {
        match tier {
            SubscriptionTier::Free => UserLimits {
                api_calls_per_day: 1000,
                concurrent_sessions: 2,
                data_retention_days: 30,
                export_limit_per_month: 5,
            },
            SubscriptionTier::Premium => UserLimits {
                api_calls_per_day: 10000,
                concurrent_sessions: 5,
                data_retention_days: 90,
                export_limit_per_month: 50,
            },
            SubscriptionTier::Enterprise => UserLimits {
                api_calls_per_day: u32::MAX, // Unlimited
                concurrent_sessions: 20,
                data_retention_days: 365,
                export_limit_per_month: u32::MAX, // Unlimited
            },
        }
    }
}

/// User identity information
#[derive(Debug, Clone)]
pub struct UserIdentityInfo {
    pub user_id: String,
    pub firebase_uid: Option<String>,
    pub email: String,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub subscription_tier: Option<String>,
}

/// User subscription information
#[derive(Debug, Clone)]
pub struct UserSubscriptionInfo {
    pub tier: SubscriptionTier,
    pub is_active: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub features: Vec<String>,
    pub limits: UserLimits,
}

/// Subscription tiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionTier {
    Free,
    Premium,
    Enterprise,
}

/// User limits based on subscription
#[derive(Debug, Clone)]
pub struct UserLimits {
    pub api_calls_per_day: u32,
    pub concurrent_sessions: u32,
    pub data_retention_days: u32,
    pub export_limit_per_month: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::entities::user_permission::UserPermission;
    
    // Mock repositories for testing
    struct MockUserRepository;
    struct MockPermissionRepository;
    
    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_user_by_id(&self, id: i32) -> Result<Option<LegacyUser>, Box<dyn std::error::Error + Send + Sync>> {
            if id == 123 {
                Ok(Some(LegacyUser {
                    id: Some(123),
                    firebase_uid: Some("test_firebase_uid".to_string()),
                    email: "test@example.com".to_string(),
                    email_verified: Some(true),
                    created_at: chrono::Utc::now(),
                    subscription_tier: Some("premium".to_string()),
                    ..Default::default()
                }))
            } else {
                Ok(None)
            }
        }
        
        async fn find_user_by_email(&self, email: String) -> Result<Option<LegacyUser>, Box<dyn std::error::Error + Send + Sync>> {
            if email == "test@example.com" {
                Ok(Some(LegacyUser {
                    id: Some(123),
                    firebase_uid: Some("test_firebase_uid".to_string()),
                    email: email,
                    email_verified: Some(true),
                    created_at: chrono::Utc::now(),
                    subscription_tier: Some("premium".to_string()),
                    ..Default::default()
                }))
            } else {
                Ok(None)
            }
        }
        
        async fn find_user_by_firebase_uid(&self, firebase_uid: String) -> Result<Option<LegacyUser>, Box<dyn std::error::Error + Send + Sync>> {
            if firebase_uid == "test_firebase_uid" {
                Ok(Some(LegacyUser {
                    id: Some(123),
                    firebase_uid: Some(firebase_uid),
                    email: "test@example.com".to_string(),
                    email_verified: Some(true),
                    created_at: chrono::Utc::now(),
                    subscription_tier: Some("premium".to_string()),
                    ..Default::default()
                }))
            } else {
                Ok(None)
            }
        }
        
        async fn update_last_login(&self, _id: i32, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        
        // Other required methods with placeholder implementations
        async fn create_user(&self, _user: LegacyUser) -> Result<LegacyUser, Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
        
        async fn update_user(&self, _user: LegacyUser) -> Result<LegacyUser, Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
        
        async fn delete_user(&self, _id: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
    }
    
    #[async_trait]
    impl UserPermissionRepository for MockPermissionRepository {
        async fn get_user_permissions(&self, user_id: i32) -> Result<Vec<UserPermission>, Box<dyn std::error::Error + Send + Sync>> {
            if user_id == 123 {
                Ok(vec![
                    UserPermission {
                        id: Some(1),
                        user_id,
                        permission_name: "epsx:analytics:read".to_string(),
                        granted_at: chrono::Utc::now(),
                        granted_by: Some(1),
                        expires_at: None,
                    },
                    UserPermission {
                        id: Some(2),
                        user_id,
                        permission_name: "epsx:trading:execute".to_string(),
                        granted_at: chrono::Utc::now(),
                        granted_by: Some(1),
                        expires_at: None,
                    },
                ])
            } else {
                Ok(vec![])
            }
        }
        
        // Other required methods with placeholder implementations
        async fn grant_permission(&self, user_id: i32, _permission: String, _granted_by: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
        
        async fn revoke_permission(&self, user_id: i32, _permission: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
        
        async fn has_permission(&self, user_id: i32, _permission: String) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            unimplemented!()
        }
    }
    
    #[tokio::test]
    async fn test_getuser_identity() {
        let user_repo = Arc::new(MockUserRepository);
        let permission_repo = Arc::new(MockPermissionRepository);
        
        let adapter = UserIdentityServiceAdapter::new(user_repo, permission_repo);
        
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        let result = adapter.getuser_identity(&user_id).await;
        assert!(result.is_ok());
        
        let identity = result.unwrap();
        assert_eq!(identity.user_id, "123");
        assert_eq!(identity.email, "test@example.com");
        assert_eq!(identity.subscription_tier, Some("premium".to_string()));
    }
    
    #[tokio::test]
    async fn test_get_user_permissions() {
        let user_repo = Arc::new(MockUserRepository);
        let permission_repo = Arc::new(MockPermissionRepository);
        
        let adapter = UserIdentityServiceAdapter::new(user_repo, permission_repo);
        
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        let result = adapter.get_user_permissions(&user_id).await;
        assert!(result.is_ok());
        
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&"epsx:analytics:read".to_string()));
        assert!(permissions.contains(&"epsx:trading:execute".to_string()));
    }
    
    #[tokio::test]
    async fn test_validate_user_access() {
        let user_repo = Arc::new(MockUserRepository);
        let permission_repo = Arc::new(MockPermissionRepository);
        
        let adapter = UserIdentityServiceAdapter::new(user_repo, permission_repo);
        
        let user_id = AuthenticatedUserId::from_verified_user(
            crate::domain::shared_kernel::value_objects::UserId::new("123".to_string())
        );
        
        // Test exact permission match
        let result = adapter.validate_user_access(&user_id, "epsx:analytics:read").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test permission user doesn't have
        let result = adapter.validate_user_access(&user_id, "epsx:admin:manage").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}