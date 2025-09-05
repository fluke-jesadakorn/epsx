use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use std::str::FromStr;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::shared_kernel::{DomainResult, DomainError};
use crate::domain::user_management::{
    UserRepositoryPort, User, UserId, Email, FirebaseUid, Permission
};
use crate::domain::user_management::{UserSearchCriteria, UserSearchResult};
use crate::infra::db::diesel::{
    DbPool,
    schema::{users, user_permissions},
    models::{DieselUser, NewDieselUser}
};
use crate::infrastructure::adapters::repositories::mappers::UserMapper;

/// Concrete implementation of UserRepositoryPort using Diesel ORM
pub struct UserRepositoryAdapter {
    pool: Arc<DbPool>,
}

impl UserRepositoryAdapter {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
    
    /// Load user with their permissions from database
    async fn load_user_with_permissions(&self, user_uuid: &Uuid) -> DomainResult<Option<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
                    user_permissions::created_at.eq(chrono::Utc::now()),
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
        UserId::from_string(&uuid.to_string()).map_err(DomainError::from)
    }
    
    async fn find_by_id(&self, id: &UserId) -> DomainResult<Option<User>> {
        let user_uuid = Uuid::from_str(&id.to_string())
            .map_err(|e| DomainError::validation_error("user_id", &e.to_string()))?;
        
        self.load_user_with_permissions(&user_uuid).await
    }
    
    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        // Find users with specific permission
        let user_uuids: Vec<Uuid> = user_permissions::table
            .filter(user_permissions::permission.eq(permission.to_string()))
            .select(user_permissions::user_id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        let mut users = Vec::new();
        for user_uuid in user_uuids {
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
        // Simple implementation - can be enhanced
        let users = Vec::new();
        let total_count = 0;
        Ok(UserSearchResult::new(users, total_count, offset, limit))
    }
    
    async fn count_by_criteria(&self, _criteria: &UserSearchCriteria) -> DomainResult<u64> {
        // Simple implementation
        Ok(0)
    }
    
    async fn find_eligible_for_auto_assignment(&self) -> DomainResult<Vec<User>> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
        // Find active users
        let diesel_users = users::table
            .filter(users::is_active.eq(Some(true)))
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
            self.save(user).await?;
        }
        Ok(())
    }
    
    async fn health_check(&self) -> DomainResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| DomainError::invalid_operation(format!("Database operation failed: {}", e), "UserRepository"))?;
        
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