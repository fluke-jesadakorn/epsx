use async_trait::async_trait;
use crate::domain::shared_kernel::value_objects::UserId;
use std::sync::Arc;
use uuid::Uuid;
use std::str::FromStr;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::shared_kernel::{DomainResult, DomainError};
use crate::domain::user_management::{
    UserRepositoryPort, User, Email, FirebaseUid, Permission
};
use crate::domain::user_management::{UserSearchCriteria, UserSearchResult};
use crate::infrastructure::adapters::repositories::diesel::{
    DbPool,
    schema::{users, user_permissions},
    models::DieselUser
};
use crate::infrastructure::adapters::repositories::mappers::UserMapper;

// Legacy compatibility imports
use crate::application::ports::outbound::repository_ports::UserRepository;
use crate::domain::user_management::aggregates::user::User as DddUser;

/// Concrete implementation of UserRepositoryPort using Diesel ORM
pub struct UserRepositoryAdapter {
    pool: Arc<DbPool>,
}

unsafe impl Send for UserRepositoryAdapter {}
unsafe impl Sync for UserRepositoryAdapter {}

impl UserRepositoryAdapter {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    /// Load user with their permissions from database
    async fn load_user_with_permissions(&self, user_uuid: &Uuid) -> DomainResult<Option<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Load user data
        let diesel_user = users::table
            .filter(users::id.eq(user_uuid))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        match diesel_user {
            Some(diesel_user) => {
                // Load user permissions
                let permission_strings: Vec<String> = user_permissions::table
                    .filter(user_permissions::user_id.eq(user_uuid))
                    .select(user_permissions::permission)
                    .load::<String>(&mut conn)
                    .await
                    .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
                
                // Convert to domain aggregate
                let user = UserMapper::to_domain(diesel_user, permission_strings)?;
                Ok(Some(user))
            }
            None => Ok(None)
        }
    }
    
    /// Save user permissions to database
    async fn save_user_permissions(&self, user: &User) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        let user_uuid = Uuid::from_str(&user.id().to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Delete existing permissions
        diesel::delete(user_permissions::table.filter(user_permissions::user_id.eq(&user_uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        // Insert new permissions
        let permissions = UserMapper::extract_permissions(user);
        for permission in permissions {
            diesel::insert_into(user_permissions::table)
                .values((
                    user_permissions::user_id.eq(&user_uuid),
                    user_permissions::permission.eq(permission),
                    user_permissions::granted_at.eq(Some(chrono::Utc::now())),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        }
        
        Ok(())
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
        
        // Try to parse as UUID first (for database IDs)
        if let Ok(user_uuid) = Uuid::from_str(&id_str) {
            return self.load_user_with_permissions(&user_uuid).await;
        }
        
        // If not a UUID, treat as Firebase UID and look up by firebase_uid field
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Find user by Firebase UID
        let diesel_user = users::table
            .filter(users::firebase_uid.eq(&id_str))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        match diesel_user {
            Some(diesel_user) => {
                self.load_user_with_permissions(&diesel_user.id).await
            }
            None => Ok(None)
        }
    }
    
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Find user by email
        let diesel_user = users::table
            .filter(users::email.eq(email.to_string()))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        match diesel_user {
            Some(diesel_user) => {
                self.load_user_with_permissions(&diesel_user.id).await
            }
            None => Ok(None)
        }
    }
    
    async fn find_by_firebase_uid(&self, firebase_uid: &FirebaseUid) -> DomainResult<Option<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Find user by Firebase UID
        let diesel_user = users::table
            .filter(users::firebase_uid.eq(firebase_uid.to_string()))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        match diesel_user {
            Some(diesel_user) => {
                self.load_user_with_permissions(&diesel_user.id).await
            }
            None => Ok(None)
        }
    }
    
    async fn save(&self, user: &User) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        let user_uuid = Uuid::from_str(&user.id().to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Check if user exists
        let exists = users::table
            .filter(users::id.eq(&user_uuid))
            .select(DieselUser::as_select())
            .first::<DieselUser>(&mut conn)
            .await
            .optional()
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?
            .is_some();
        
        if exists {
            // Update existing user
            let update_model = UserMapper::to_update_diesel(user);
            diesel::update(users::table.filter(users::id.eq(&user_uuid)))
                .set(&update_model)
                .execute(&mut conn)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        } else {
            // Insert new user
            let new_model = UserMapper::to_new_diesel(user)?;
            diesel::insert_into(users::table)
                .values(&new_model)
                .execute(&mut conn)
                .await
                .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        }
        
        // Save permissions
        self.save_user_permissions(user).await?;
        
        Ok(())
    }
    
    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        let user_uuid = Uuid::from_str(&id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        // Delete user permissions first (foreign key constraint)
        diesel::delete(user_permissions::table.filter(user_permissions::user_id.eq(&user_uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        // Delete user
        diesel::delete(users::table.filter(users::id.eq(&user_uuid)))
            .execute(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        Ok(())
    }
    
    async fn find_by_permission(&self, permission: &Permission) -> DomainResult<Vec<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Find users with specific permission
        let user_id_uuids: Vec<uuid::Uuid> = user_permissions::table
            .filter(user_permissions::permission.eq(permission.to_string()))
            .select(user_permissions::user_id)
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        let mut users = Vec::new();
        for user_uuid in user_id_uuids {
            if let Some(user) = self.load_user_with_permissions(&user_uuid).await? {
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
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Helper function to build query with filters
        let build_query = || {
            let mut query = users::table.into_boxed();
            
            // Apply search term filter
            if let Some(search_term) = &criteria.search_term {
                if !search_term.is_empty() {
                    query = query.filter(users::email.ilike(format!("%{}%", search_term)));
                }
            }
            
            // Apply email pattern filter
            if let Some(email_pattern) = &criteria.email_pattern {
                if !email_pattern.is_empty() {
                    let pattern = email_pattern.replace('*', "%");
                    query = query.filter(users::email.ilike(pattern));
                }
            }
            
            // Apply active status filter
            if let Some(is_active) = criteria.is_active {
                query = query.filter(users::is_active.eq(is_active));
            }
            
            // Apply email verification filter
            if let Some(email_verified) = criteria.email_verified {
                query = query.filter(users::email_verified.eq(email_verified));
            }
            
            query
        };
        
        // Count total matching users for pagination
        let total_count: i64 = build_query()
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Count query failed: {}", e), "UserRepository"))?;
        
        // Get paginated users
        let diesel_users: Vec<DieselUser> = build_query()
            .limit(limit.into())
            .offset(offset.into())
            .select(DieselUser::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database query failed: {}", e), "UserRepository"))?;
        
        // Load each user with their permissions
        let mut users = Vec::new();
        for diesel_user in diesel_users {
            if let Some(user) = self.load_user_with_permissions(&diesel_user.id).await? {
                users.push(user);
            }
        }
        
        tracing::info!("Found {} users out of {} total matching criteria", users.len(), total_count);
        
        Ok(UserSearchResult::new(users, total_count as u64, offset, limit))
    }
    
    async fn count_by_criteria(&self, criteria: &UserSearchCriteria) -> DomainResult<u64> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Build query with filters (similar to find_by_criteria)
        let mut query = users::table.into_boxed();
        
        // Apply search term filter
        if let Some(search_term) = &criteria.search_term {
            if !search_term.is_empty() {
                query = query.filter(users::email.ilike(format!("%{}%", search_term)));
            }
        }
        
        // Apply email pattern filter
        if let Some(email_pattern) = &criteria.email_pattern {
            if !email_pattern.is_empty() {
                let pattern = email_pattern.replace('*', "%");
                query = query.filter(users::email.ilike(pattern));
            }
        }
        
        // Apply active status filter
        if let Some(is_active) = criteria.is_active {
            query = query.filter(users::is_active.eq(is_active));
        }
        
        // Apply email verification filter
        if let Some(email_verified) = criteria.email_verified {
            query = query.filter(users::email_verified.eq(email_verified));
        }
        
        // Count total matching users
        let count: i64 = query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Count query failed: {}", e), "UserRepository"))?;
        
        Ok(count as u64)
    }
    
    async fn find_eligible_for_auto_assignment(&self) -> DomainResult<Vec<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Find active users using the actual is_active field
        let diesel_users = users::table
            .filter(users::is_active.eq(true))
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        let mut users = Vec::new();
        for diesel_user in diesel_users {
            if let Some(user) = self.load_user_with_permissions(&diesel_user.id).await? {
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
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database connection failed: {}", e), "UserRepository"))?;
        
        // Simple health check
        let _ = users::table
            .limit(1)
            .select(DieselUser::as_select())
            .load::<DieselUser>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        Ok(())
    }
    
    async fn cleanup_expired_permissions(&self) -> DomainResult<u32> {
        // Simple implementation - would need to implement permission expiry logic
        Ok(0)
    }
}

// Simple error type for legacy repository compatibility
#[derive(Debug)]
pub struct LegacyRepositoryError(String);

impl std::fmt::Display for LegacyRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LegacyRepositoryError {}

// Legacy UserRepository trait implementation for compatibility
#[async_trait]
impl UserRepository for UserRepositoryAdapter {
    type Error = LegacyRepositoryError;

    async fn find_by_id(&self, user_id: &crate::domain::shared_kernel::value_objects::UserId) -> Result<Option<DddUser>, Self::Error> {
        match UserRepositoryPort::find_by_id(self, user_id).await {
            Ok(user_opt) => Ok(user_opt),
            Err(e) => Err(LegacyRepositoryError(e.to_string())),
        }
    }

    async fn find_by_email(&self, email: &crate::domain::shared_kernel::value_objects::Email) -> Result<Option<DddUser>, Self::Error> {
        // Convert legacy Email to DDD Email
        let ddd_email = crate::domain::user_management::value_objects::Email::new(email.to_string())
            .map_err(|e| LegacyRepositoryError(format!("Email conversion failed: {}", e)))?;
            
        match UserRepositoryPort::find_by_email(self, &ddd_email).await {
            Ok(user_opt) => Ok(user_opt),
            Err(e) => Err(LegacyRepositoryError(e.to_string())),
        }
    }

    async fn save(&self, user: &DddUser) -> Result<(), Self::Error> {
        match UserRepositoryPort::save(self, user).await {
            Ok(()) => Ok(()),
            Err(e) => Err(LegacyRepositoryError(e.to_string())),
        }
    }

    async fn delete(&self, user_id: &crate::domain::shared_kernel::value_objects::UserId) -> Result<(), Self::Error> {
        match UserRepositoryPort::delete(self, user_id).await {
            Ok(()) => Ok(()),
            Err(e) => Err(LegacyRepositoryError(e.to_string())),
        }
    }

    async fn list_users(&self, _offset: usize, _limit: usize) -> Result<Vec<DddUser>, Self::Error> {
        // For now, return empty list - would need to implement pagination
        Ok(Vec::new())
    }
}