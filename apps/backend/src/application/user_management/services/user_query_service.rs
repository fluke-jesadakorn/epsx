use std::sync::Arc;

use crate::application::shared::{ApplicationResult, ApplicationError};
use crate::domain::user_management::UserSearchCriteria;
use crate::application::user_management::{
    GetUserByFirebaseUidQuery,
    GetUserByFirebaseUidResponse,
    ListUsersQuery,
    ListUsersResponse,
    UserSummary,
};
use crate::domain::user_management::UserRepositoryPort;
use crate::domain::shared_kernel::AggregateRoot;

/// User Query Service
/// Handles read-only operations for user data
#[derive(Clone)]
pub struct UserQueryService {
    user_repository: Arc<dyn UserRepositoryPort>,
}

impl UserQueryService {
    /// Create a new UserQueryService
    pub fn new(user_repository: Arc<dyn UserRepositoryPort>) -> Self {
        Self { 
            user_repository,
        }
    }
    
    /// Get user by Firebase UID
    pub async fn get_user_by_firebase_uid(
        &self,
        query: GetUserByFirebaseUidQuery,
    ) -> ApplicationResult<GetUserByFirebaseUidResponse> {
        tracing::info!("Processing GetUserByFirebaseUidQuery for firebase_uid: {}", query.firebase_uid.to_string());
        
        let user = self.user_repository
            .find_by_firebase_uid(&query.firebase_uid)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("User", query.firebase_uid.to_string()))?;
        
        Ok(GetUserByFirebaseUidResponse {
            firebase_uid: user.firebase_uid().clone(),
            email: user.email().clone(),
            email_verified: user.is_email_verified(),
            is_active: user.is_active(),
            permissions: user.permissions().clone(),
        })
    }
    
    /// List users with filtering and pagination
    pub async fn list_users(
        &self,
        query: ListUsersQuery,
    ) -> ApplicationResult<ListUsersResponse> {
        tracing::info!("Processing ListUsersQuery with limit: {}, offset: {}", query.limit, query.offset);
        
        // Create search criteria from query
        let mut search_criteria = UserSearchCriteria::default();
        
        // Apply email domain filter if specified
        if let Some(email_domain) = &query.email_domain_filter {
            search_criteria.email_pattern = Some(format!("*@{}", email_domain));
        }
        
        // Apply permission filter if specified
        if let Some(permissions) = &query.permission_filter {
            for permission_str in permissions {
                if let Ok(permission) = crate::domain::user_management::value_objects::Permission::new(permission_str.clone()) {
                    search_criteria.has_permissions.push(permission);
                }
            }
        }
        
        let search_result = self.user_repository
            .find_by_criteria(&search_criteria, query.limit as u32, query.offset as u32)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;
        
        let users = search_result.users;
        let total_count = search_result.total_count as usize;
        
        // Convert domain users to summary format using pure domain data
        let mut user_summaries = Vec::new();
        
        for user in users {
            let permissions_vec: Vec<String> = user.permissions().iter()
                .map(|p| p.to_string())
                .collect();
            
            // Derive role from permissions
            let role = if permissions_vec.iter().any(|p| p.contains("admin")) {
                "admin".to_string()
            } else if permissions_vec.iter().any(|p| p.contains("premium")) {
                "premium".to_string()
            } else {
                "user".to_string()
            };
            
            // Derive status from is_active
            let status = if user.is_active() {
                "active".to_string()
            } else {
                "inactive".to_string()
            };
            
            user_summaries.push(UserSummary {
                id: user.firebase_uid().to_string(),  // Use firebase_uid as id
                firebase_uid: user.firebase_uid().clone(),
                email: user.email().clone(),
                display_name: None, // Domain User doesn't expose display_name directly
                role,
                status,
                is_active: user.is_active(),
                email_verified: user.is_email_verified(),
                permissions: user.permissions().clone(),
                package_tier: "free".to_string(), // Default tier - should be derived from permissions
                created_at: AggregateRoot::created_at(&user),
                updated_at: AggregateRoot::updated_at(&user),
                last_login_at: user.last_login_at(),
            });
        }
        
        Ok(ListUsersResponse::new(user_summaries, total_count))
    }

    /// Update user (placeholder)
    pub async fn update_user(
        &self,
        _user_id: &str,
        _update_data: UserUpdateData,
    ) -> ApplicationResult<()> {
        // TODO: Implement user update with SQLx
        Ok(())
    }

    /// Delete user (placeholder)
    pub async fn delete_user(&self, _user_id: &str) -> ApplicationResult<()> {
        // TODO: Implement user deletion with SQLx
        Ok(())
    }

    /// Create user (placeholder)
    pub async fn create_user(&self, _user_data: CreateUserData) -> ApplicationResult<String> {
        // TODO: Implement user creation with SQLx
        Ok("user_id_placeholder".to_string())
    }
}

/// Placeholder structs for user operations
#[derive(Debug)]
pub struct UserUpdateData {
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug)]
pub struct CreateUserData {
    pub email: String,
    pub firebase_uid: String,
}