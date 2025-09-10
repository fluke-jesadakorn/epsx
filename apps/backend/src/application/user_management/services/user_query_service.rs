use std::sync::Arc;
use uuid::Uuid;
use std::str::FromStr;

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

// Import database access for enhanced UserSummary mapping
use crate::infrastructure::adapters::repositories::diesel::{DbPool, schema::users, models::DieselUser};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

/// User Query Service
/// Handles read-only operations for user data
#[derive(Clone)]
pub struct UserQueryService {
    user_repository: Arc<dyn UserRepositoryPort>,
    db_pool: Arc<DbPool>,
}

impl UserQueryService {
    /// Create a new UserQueryService
    pub fn new(user_repository: Arc<dyn UserRepositoryPort>, db_pool: Arc<DbPool>) -> Self {
        Self { 
            user_repository,
            db_pool,
        }
    }

    /// Get enhanced user data from database for UserSummary
    async fn get_user_database_info(&self, firebase_uid: &str) -> ApplicationResult<Option<DieselUser>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| ApplicationError::infrastructure(format!("Database connection failed: {}", e)))?;
        
        let diesel_user = users::table
            .filter(users::firebase_uid.eq(firebase_uid))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::infrastructure(format!("Database query failed: {}", e)))?;
        
        Ok(diesel_user)
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
        
        // Convert domain users to summary format with enhanced database fields
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

            // Get enhanced database info for this user
            let database_info = self.get_user_database_info(user.firebase_uid().to_string().as_str()).await?;
            
            user_summaries.push(UserSummary {
                id: user.firebase_uid().to_string(),  // Use firebase_uid as id
                firebase_uid: user.firebase_uid().clone(),
                email: user.email().clone(),
                display_name: database_info.as_ref()
                    .and_then(|db_user| db_user.display_name.clone()), // Get from database
                role,
                status,
                is_active: user.is_active(),
                email_verified: user.is_email_verified(),
                permissions: user.permissions().clone(),
                package_tier: database_info.as_ref()
                    .and_then(|db_user| db_user.package_tier.clone())
                    .unwrap_or_else(|| "free".to_string()), // Get from database with fallback
                created_at: AggregateRoot::created_at(&user),
                updated_at: AggregateRoot::updated_at(&user),
                last_login_at: user.last_login_at(),
            });
        }
        
        Ok(ListUsersResponse::new(user_summaries, total_count))
    }
}