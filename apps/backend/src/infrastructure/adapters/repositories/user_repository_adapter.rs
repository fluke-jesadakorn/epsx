use async_trait::async_trait;
use crate::domain::shared_kernel::value_objects::UserId;
use std::sync::Arc;
use uuid::Uuid;
use std::str::FromStr;

use sqlx::{PgPool, Row};

use crate::domain::shared_kernel::{DomainResult, DomainError};
use crate::domain::user_management::{
    UserRepositoryPort, User, Email, Permission
};
use crate::domain::user_management::{UserSearchCriteria, UserSearchResult};
use crate::infrastructure::adapters::repositories::{DbPool, SqlxBaseRepository};

/// UserRepositoryPort implementation using SQLx for Cloud Run compatibility
/// Safe Send/Sync implementation - SqlxBaseRepository contains Arc<PgPool> which is Send+Sync
#[derive(Clone)]
pub struct UserRepositoryAdapter {
    base: SqlxBaseRepository,
}

// Safe implementation - SqlxBaseRepository wraps Arc<PgPool> which is Send+Sync
// This is safe because:
// 1. SqlxBaseRepository contains Arc<PgPool> 
// 2. Arc<T> is Send+Sync when T is Send+Sync
// 3. PgPool is Send+Sync by design
// No unsafe blocks needed - Rust's type system handles this correctly

impl UserRepositoryAdapter {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { 
            base: SqlxBaseRepository::new(pool),
        }
    }
    
    fn get_pool(&self) -> &PgPool {
        self.base.get_pool()
    }
    
    /// Load user with permissions from database
    async fn load_user_with_permissions(&self, user_uuid: &Uuid) -> DomainResult<Option<User>> {
        let pool = self.get_pool();
        
        // Load user data from database
        let user_row = sqlx::query(
            "SELECT id, email, created_at, updated_at, is_active, role, subscription_tier FROM users WHERE id = $1"
        )
        .bind(user_uuid)
        .fetch_optional(pool)
        .await
        .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        match user_row {
            Some(user_row) => {
                // Set permissions based on user role
                let role: Option<String> = user_row.try_get("role").unwrap_or(None);
                let permission_strings: Vec<String> = match role.as_deref() {
                    Some("admin") => vec!["admin:*:*".to_string()],
                    Some("user") => vec!["epsx:basic:read".to_string()],
                    _ => vec!["epsx:basic:read".to_string()], // Default permission
                };
                
                // Convert to domain aggregate
                let user = self.create_domain_user(
                    user_row.get("id"),
                    user_row.get("email"),
                    None, // No display_name in actual schema
                    permission_strings
                )?;
                Ok(Some(user))
            }
            None => Ok(None)
        }
    }
    
    /// Convert database fields to domain User
    fn create_domain_user(
        &self, 
        id: Uuid,
        email: String,
        _display_name: Option<String>,
        permissions: Vec<String>
    ) -> DomainResult<User> {
        let user_id = UserId::from_string(id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        let user_email = Email::new(email)
            .map_err(|e| DomainError::validation_error("email", &e.to_string()))?;
        
        let _domain_permissions: Vec<Permission> = permissions
            .into_iter()
            .filter_map(|p| Permission::new(p).ok())
            .collect();
        
        // For migration purposes, create a placeholder wallet address
        // In a real system, this would need to be properly handled
        let wallet_address = crate::domain::user_management::value_objects::WalletAddress::new("0x0000000000000000000000000000000000000000".to_string())
            .map_err(|e| DomainError::validation_error("wallet_address", &e.to_string()))?;
        
        User::create(
            user_id,
            wallet_address,
            user_email
        ).map_err(DomainError::from)
    }
    
    /// Save user permissions by updating role
    async fn save_user_permissions(&self, user: &User) -> DomainResult<()> {
        // Update user role based on permissions
        let pool = self.get_pool();
        
        let user_uuid = Uuid::from_str(&user.id().to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Determine role based on permissions
        let permissions = self.extract_permissions(user);
        let role = if permissions.iter().any(|p| p.contains("admin")) {
            "admin"
        } else {
            "user"
        };
        
        // Update user role
        sqlx::query("UPDATE users SET role = $1, updated_at = $2 WHERE id = $3")
            .bind(role)
            .bind(chrono::Utc::now())
            .bind(user_uuid)
            .execute(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        Ok(())
    }
    
    /// Extract permissions from User
    fn extract_permissions(&self, user: &User) -> Vec<String> {
        user.permissions().iter().map(|p| p.to_string()).collect()
    }
}

#[async_trait]
impl UserRepositoryPort for UserRepositoryAdapter {
    async fn next_identity(&self) -> DomainResult<UserId> {
        let uuid = Uuid::new_v4();
        UserId::from_string(uuid.to_string()).map_err(DomainError::from)
    }
    
    async fn find_by_id(&self, id: &UserId) -> DomainResult<Option<User>> {
        let id_str = id.to_string();
        
        // Try to parse as UUID (for database IDs)
        if let Ok(user_uuid) = Uuid::from_str(&id_str) {
            return self.load_user_with_permissions(&user_uuid).await;
        }
        
        // If not a UUID, return None since we no longer support Firebase UID lookup
        Ok(None)
    }
    
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        let pool = self.get_pool();
        
        // Find user by email - this is the critical query that was timing out!
        tracing::debug!("Looking up user by email: {}", email.to_string());
        
        let user_row = sqlx::query!(
            "SELECT id FROM users WHERE email = $1",
            email.to_string()
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("SQLx query failed for email {}: {}", email.to_string(), e);
            DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository")
        })?;
        
        match user_row {
            Some(row) => {
                tracing::debug!("Found user with ID: {} for email: {}", row.id, email.to_string());
                self.load_user_with_permissions(&row.id).await
            }
            None => {
                tracing::debug!("No user found for email: {}", email.to_string());
                Ok(None)
            }
        }
    }
    
    // Firebase UID lookup removed - migrated to Web3
    
    async fn save(&self, user: &User) -> DomainResult<()> {
        let pool = self.get_pool();
        
        let user_uuid = Uuid::from_str(&user.id().to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Check if user exists
        let exists = sqlx::query("SELECT id FROM users WHERE id = $1")
            .bind(user_uuid)
            .fetch_optional(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?
            .is_some();
        
        if exists {
            // Update existing user - using actual schema
            sqlx::query(
                "UPDATE users SET email = $2, is_active = $3, updated_at = $4 WHERE id = $1"
            )
            .bind(user_uuid)
            .bind(user.email().to_string())
            .bind(true) // is_active
            .bind(chrono::Utc::now())
            .execute(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        } else {
            // Insert new user - using actual schema
            sqlx::query(
                "INSERT INTO users (id, firebase_uid, email, is_active, role, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(user_uuid)
            .bind(user.id().to_string()) // Use user_id as firebase_uid for backward compatibility
            .bind(user.email().to_string())
            .bind(true) // is_active
            .bind("user") // default role
            .bind(chrono::Utc::now())
            .bind(chrono::Utc::now())
            .execute(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        }
        
        // Save permissions
        self.save_user_permissions(user).await?;
        
        Ok(())
    }
    
    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        let pool = self.get_pool();
        
        let user_uuid = Uuid::from_str(&id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Delete user directly since user_permissions table doesn't exist yet
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_uuid)
            .execute(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        Ok(())
    }
    
    async fn find_by_permission(&self, permission: &Permission) -> DomainResult<Vec<User>> {
        let pool = self.get_pool();
        
        // Find users with specific permission - simplified since user_permissions table doesn't exist
        // For now, find users by role that matches the permission
        let role_filter = if permission.to_string().contains("admin") {
            "admin"
        } else {
            "user"
        };
        
        let user_rows = sqlx::query("SELECT id FROM users WHERE role = $1")
            .bind(role_filter)
            .fetch_all(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        let mut users = Vec::new();
        for row in user_rows {
            let user_id: Uuid = row.get("id");
            if let Some(user) = self.load_user_with_permissions(&user_id).await? {
                users.push(user);
            }
        }
        
        Ok(users)
    }
    
    async fn find_by_criteria(
        &self, 
        criteria: &UserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> DomainResult<UserSearchResult> {
        let pool = self.get_pool();
        
        // Count query - simplified approach for search term
        let count_query = if criteria.search_term.is_some() && !criteria.search_term.as_ref().unwrap().is_empty() {
            "SELECT COUNT(*) as count FROM users WHERE email ILIKE $1".to_string()
        } else {
            "SELECT COUNT(*) as count FROM users".to_string()
        };
        let total_count = if criteria.search_term.is_some() && !criteria.search_term.as_ref().unwrap().is_empty() {
            let search_term = criteria.search_term.as_ref().unwrap();
            let search_pattern = format!("%{}%", search_term);
            sqlx::query_scalar::<_, i64>(&count_query)
                .bind(search_pattern)
                .fetch_one(pool)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Count query failed: {}", e), "UserRepository"))? as u64
        } else {
            sqlx::query_scalar::<_, i64>(&count_query)
                .fetch_one(pool)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Count query failed: {}", e), "UserRepository"))? as u64
        };
        
        // For simplicity, let's use a basic search without complex dynamic parameters
        let user_rows = if criteria.search_term.is_some() && !criteria.search_term.as_ref().unwrap().is_empty() {
            let search_term = criteria.search_term.as_ref().unwrap();
            let search_pattern = format!("%{}%", search_term);
            sqlx::query("SELECT id FROM users WHERE email ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(search_pattern)
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(pool).await
        } else {
            sqlx::query("SELECT id FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(limit as i64)
                .bind(offset as i64)
                .fetch_all(pool).await
        }.map_err(|e| DomainError::invalid_operation(format!("Database query failed: {}", e), "UserRepository"))?;
        
        // Load each user with their permissions
        let mut users = Vec::new();
        for row in user_rows {
            let user_id: uuid::Uuid = row.get("id");
            if let Some(user) = self.load_user_with_permissions(&user_id).await? {
                users.push(user);
            }
        }
        
        tracing::info!("Found {} users out of {} total matching criteria", users.len(), total_count);
        
        Ok(UserSearchResult::new(users, total_count, offset, limit))
    }
    
    async fn count_by_criteria(&self, criteria: &UserSearchCriteria) -> DomainResult<u64> {
        let pool = self.get_pool();
        
        // Simplified count query - for basic search functionality
        let count = if criteria.search_term.is_some() && !criteria.search_term.as_ref().unwrap().is_empty() {
            let search_term = criteria.search_term.as_ref().unwrap();
            let search_pattern = format!("%{}%", search_term);
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email ILIKE $1")
                .bind(search_pattern)
                .fetch_one(pool)
                .await
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(pool)
                .await
        }.map_err(|e| DomainError::invalid_operation(format!("Count query failed: {}", e), "UserRepository"))?;
        
        Ok(count as u64)
    }
    
    async fn find_eligible_for_auto_assignment(&self) -> DomainResult<Vec<User>> {
        let pool = self.get_pool();
        
        // Find active users using the actual is_active field
        let user_rows = sqlx::query("SELECT id FROM users WHERE is_active = true")
            .fetch_all(pool)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        let mut users = Vec::new();
        for row in user_rows {
            let user_id: Uuid = row.get("id");
            if let Some(user) = self.load_user_with_permissions(&user_id).await? {
                users.push(user);
            }
        }
        
        Ok(users)
    }
    
    async fn save_batch(&self, users: &[User]) -> DomainResult<()> {
        for user in users {
            UserRepositoryPort::save(self, user).await?;
        }
        Ok(())
    }
    
    async fn health_check(&self) -> DomainResult<()> {
        self.base.health_check_impl().await
    }
    
    async fn cleanup_expired_permissions(&self) -> DomainResult<u32> {
        Ok(0)
    }
}