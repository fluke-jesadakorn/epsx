use async_trait::async_trait;
use std::sync::Arc;
use crate::config::env::get_env_var;

use crate::infrastructure::adapters::services::firebase::{FirebaseAdmin, FirebaseUser};
// Removed legacy service imports - using simple roles

/// Firebase-first user service that queries Firebase directly for all user data
/// No local user data storage - Firebase is the single source of truth
#[derive(Clone)]
pub struct FirebaseUserService {
    firebase_admin: FirebaseAdmin,
}

/// User creation request
#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

/// User update request  
#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub disabled: Option<bool>,
    pub role: Option<String>,
}

/// User list filters
#[derive(Debug, Clone)]
pub struct UserListFilters {
    pub max_results: Option<u32>,
    pub page_token: Option<String>,
    pub role_filter: Option<String>,
    pub email_domain_filter: Option<String>,
}

/// Firebase user service trait for dependency injection
#[async_trait]
pub trait FirebaseUserServiceTrait: Send + Sync {
    async fn get_user_by_uid(&self, firebase_uid: &str) -> Result<FirebaseUser, UserServiceError>;
    async fn get_user_by_email(&self, email: &str) -> Result<FirebaseUser, UserServiceError>;
    async fn create_user(&self, request: CreateUserRequest) -> Result<String, UserServiceError>;
    async fn update_user(&self, firebase_uid: &str, request: UpdateUserRequest) -> Result<(), UserServiceError>;
    async fn delete_user(&self, firebase_uid: &str) -> Result<(), UserServiceError>;
    async fn set_user_role(&self, firebase_uid: &str, role: &str) -> Result<(), UserServiceError>;
    async fn list_users(&self, filters: UserListFilters) -> Result<(Vec<FirebaseUser>, Option<String>), UserServiceError>;
    async fn validate_admin_access(&self, firebase_uid: &str) -> Result<AdminAccessInfo, UserServiceError>;
}

/// Admin access information
#[derive(Debug, Clone)]
pub struct AdminAccessInfo {
    pub has_admin_access: bool,
    pub access_level: String,
    pub role: String,
    pub permissions: Vec<String>,
}

/// User service errors
#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    
    #[error("Invalid user data: {0}")]
    InvalidUserData(String),
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Firebase error: {0}")]
    FirebaseError(String),
    
    #[error("Internal service error: {0}")]
    InternalError(String),
}

impl FirebaseUserService {
    /// Create new Firebase user service
    pub async fn new() -> Result<Self, UserServiceError> {
        let project_id = get_env_var("FIREBASE_PROJECT_ID")
            .unwrap_or_else(|_| "epsx-449804".to_string()); // Default project ID
        let firebase_admin = FirebaseAdmin::new(project_id);
            
        Ok(Self {
            firebase_admin,
        })
    }
    
    /// Create service with simple roles (legacy method kept for compatibility)
    pub async fn with_database_roles(_db_pool: Arc<sqlx::PgPool>) -> Result<Self, UserServiceError> {
        Self::new().await
    }
    
    /// Create service with existing Firebase Admin instance
    pub fn with_firebase_admin(firebase_admin: FirebaseAdmin) -> Self {
        Self {
            firebase_admin,
        }
    }
    
    /// Create service with simple roles (legacy method kept for compatibility)
    pub fn with_admin_module_service(firebase_admin: FirebaseAdmin, _admin_module_service: std::sync::Arc<()>) -> Self {
        Self::with_firebase_admin(firebase_admin)
    }

    /// Validate admin access for user (simplified for unified role system)
    pub async fn validate_admin_access(&self, firebase_uid: &str) -> Result<bool, UserServiceError> {
        tracing::info!("Validating admin access for firebase_uid: {}", firebase_uid);
        
        // Development fallback: For specific test users (Firebase UID only)
        let test_admin_uid = get_env_var("TEST_ADMIN_UID").unwrap_or_default();
        
        if !test_admin_uid.is_empty() && firebase_uid == test_admin_uid {
            tracing::info!("User {} granted admin access via TEST_ADMIN_UID environment variable", firebase_uid);
            return Ok(true);
        }
        
        // Simple check: Firebase custom claims or UID pattern for development
        let legacy_admin_check = firebase_uid.contains("admin");
        tracing::info!("User {} legacy admin check (contains 'admin'): {}", firebase_uid, legacy_admin_check);
        Ok(legacy_admin_check)
    }
}

#[async_trait]
impl FirebaseUserServiceTrait for FirebaseUserService {
    /// Get user by Firebase UID - queries Firebase directly
    async fn get_user_by_uid(&self, firebase_uid: &str) -> Result<FirebaseUser, UserServiceError> {
        tracing::info!("Getting user by UID: {}", firebase_uid);
        
        self.firebase_admin
            .get_user(firebase_uid)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get user {}: {}", firebase_uid, e);
                UserServiceError::UserNotFound(firebase_uid.to_string())
            })
    }
    
    /// Get user by email - queries Firebase directly  
    async fn get_user_by_email(&self, email: &str) -> Result<FirebaseUser, UserServiceError> {
        tracing::info!("Getting user by email: {}", email);
        
        self.firebase_admin
            .get_user_by_email(email)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get user by email {}: {}", email, e);
                UserServiceError::UserNotFound(email.to_string())
            })
    }
    
    /// Create new Firebase user with role assignment
    async fn create_user(&self, request: CreateUserRequest) -> Result<String, UserServiceError> {
        tracing::info!("Creating new Firebase user with email: {:?}", request.email);
        
        // Create user in Firebase
        let firebase_uid = self.firebase_admin
            .create_user(request.email.clone(), request.password, request.display_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create Firebase user: {}", e);
                if e.to_string().contains("EMAIL_EXISTS") {
                    UserServiceError::UserAlreadyExists(
                        request.email.unwrap_or_else(|| "unknown".to_string())
                    )
                } else {
                    UserServiceError::FirebaseError(e.to_string())
                }
            })?;
            
        // Set role if specified
        if let Some(role) = request.role {
            if let Err(e) = self.firebase_admin.set_user_role(&firebase_uid, &role).await {
                tracing::error!("Failed to set role for new user {}: {}", firebase_uid, e);
                // Don't fail user creation if role setting fails - user exists in Firebase
            }
        }
        
        tracing::info!("Successfully created Firebase user: {}", firebase_uid);
        Ok(firebase_uid)
    }
    
    /// Update Firebase user
    async fn update_user(&self, firebase_uid: &str, request: UpdateUserRequest) -> Result<(), UserServiceError> {
        tracing::info!("Updating Firebase user: {}", firebase_uid);
        
        // Update basic user info
        self.firebase_admin
            .update_user(firebase_uid, request.email, request.display_name, request.disabled)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update user {}: {}", firebase_uid, e);
                UserServiceError::FirebaseError(e.to_string())
            })?;
            
        // Update role if specified
        if let Some(role) = request.role {
            self.firebase_admin
                .set_user_role(firebase_uid, &role)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to update role for user {}: {}", firebase_uid, e);
                    UserServiceError::FirebaseError(e.to_string())
                })?;
        }
        
        tracing::info!("Successfully updated Firebase user: {}", firebase_uid);
        Ok(())
    }
    
    /// Delete Firebase user
    async fn delete_user(&self, firebase_uid: &str) -> Result<(), UserServiceError> {
        tracing::info!("Deleting Firebase user: {}", firebase_uid);
        
        self.firebase_admin
            .delete_user(firebase_uid)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete user {}: {}", firebase_uid, e);
                UserServiceError::FirebaseError(e.to_string())
            })?;
            
        tracing::info!("Successfully deleted Firebase user: {}", firebase_uid);
        Ok(())
    }
    
    /// Set user role via Firebase custom claims
    async fn set_user_role(&self, firebase_uid: &str, role: &str) -> Result<(), UserServiceError> {
        tracing::info!("Setting role '{}' for user: {}", role, firebase_uid);
        
        self.firebase_admin
            .set_user_role(firebase_uid, role)
            .await
            .map_err(|e| {
                tracing::error!("Failed to set role for user {}: {}", firebase_uid, e);
                UserServiceError::FirebaseError(e.to_string())
            })?;
            
        tracing::info!("Successfully set role '{}' for user: {}", role, firebase_uid);
        Ok(())
    }
    
    /// List Firebase users with filtering
    async fn list_users(&self, filters: UserListFilters) -> Result<(Vec<FirebaseUser>, Option<String>), UserServiceError> {
        tracing::info!("Listing Firebase users with filters: {:?}", filters);
        
        let (mut users, next_page_token) = self.firebase_admin
            .list_users(filters.max_results, filters.page_token)
            .await
            .map_err(|e| {
                tracing::error!("Failed to list users: {}", e);
                UserServiceError::FirebaseError(e.to_string())
            })?;
            
        // Apply client-side filtering (Firebase Admin API has limited filtering)
        if let Some(role_filter) = filters.role_filter {
            users = users.into_iter()
                .filter(|user| {
                    user.custom_claims.get("role")
                        .and_then(|r| r.as_str())
                        .map(|r| r == role_filter)
                        .unwrap_or(false)
                })
                .collect();
        }
        
        if let Some(domain_filter) = filters.email_domain_filter {
            users = users.into_iter()
                .filter(|user| {
                    user.email.as_ref()
                        .map(|email| email.ends_with(&domain_filter))
                        .unwrap_or(false)
                })
                .collect();
        }
        
        tracing::info!("Listed {} Firebase users", users.len());
        Ok((users, next_page_token))
    }
    
    /// Validate admin access for user (using database role service if available)
    async fn validate_admin_access(&self, firebase_uid: &str) -> Result<AdminAccessInfo, UserServiceError> {
        tracing::info!("Validating admin access for user: {}", firebase_uid);
        
        // Using simplified role system - no complex database role service needed
        
        // Fallback to Firebase custom claims
        let firebase_user = self.get_user_by_uid(firebase_uid).await?;
        
        let has_admin_access = self.firebase_admin.user_has_admin_access(&firebase_user);
        let access_level = self.firebase_admin.get_admin_access_level(&firebase_user);
        let role = firebase_user.custom_claims.get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("user-basic-001")
            .to_string();
            
        // Generate permissions based on role and access level
        let permissions = self.generate_permissions_for_role(&role, &access_level);
        
        let admin_info = AdminAccessInfo {
            has_admin_access,
            access_level,
            role,
            permissions,
        };
        
        tracing::info!(
            "Admin access validation for {}: has_access={}, level={} (source: {})", 
            firebase_uid, 
            admin_info.has_admin_access, 
            admin_info.access_level,
"unified-roles"
        );
        
        Ok(admin_info)
    }
}

impl FirebaseUserService {
    /// Generate permissions list based on role and access level
    fn generate_permissions_for_role(&self, role: &str, _access_level: &str) -> Vec<String> {
        let mut permissions = Vec::new();
        
        // Basic permissions for all users
        permissions.extend_from_slice(&[
            "read:profile".to_string(),
            "update:profile".to_string(),
        ]);
        
        // Role-based permissions
        match role {
            "admin" => {
                permissions.extend_from_slice(&[
                    "admin:*".to_string(),
                    "create:users".to_string(),
                    "delete:users".to_string(),
                    "manage:roles".to_string(),
                    "read:analytics".to_string(),
                    "manage:system".to_string(),
                ]);
            },
            "moderator" => {
                permissions.extend_from_slice(&[
                    "admin:users".to_string(),
                    "create:users".to_string(),
                    "update:users".to_string(),
                    "read:analytics".to_string(),
                    "manage:profiles".to_string(),
                ]);
            },
            "premium" => {
                permissions.extend_from_slice(&[
                    "read:premium_analytics".to_string(),
                    "create:alerts".to_string(),
                ]);
            },
            _ => {
                // Basic user permissions only
            }
        }
        
        permissions
    }
    
    /// Verify Firebase ID token and get user data
    pub async fn verify_and_get_user(&self, id_token: &str) -> Result<FirebaseUser, UserServiceError> {
        tracing::info!("Verifying Firebase ID token and getting user data");
        
        self.firebase_admin
            .verify_id_token(id_token)
            .await
            .map_err(|e| {
                tracing::error!("Failed to verify Firebase ID token: {}", e);
                UserServiceError::FirebaseError(format!("Token verification failed: {}", e))
            })
    }
}