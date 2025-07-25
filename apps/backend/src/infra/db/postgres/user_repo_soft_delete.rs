// PostgreSQL User Repository Implementation with Soft Delete Support
// This extends the existing UserRepo with soft delete functionality

use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::app::ports::repositories::{RepoError};
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email, Role};
use super::DatabasePool;

pub struct SoftDeleteUserRepo {
    pool: DatabasePool,
}

impl SoftDeleteUserRepo {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    /// Soft delete a user (admin only functionality)
    pub async fn soft_delete(&self, id: &UserId) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "UPDATE users SET deleted_at = NOW(), updated_at = NOW() 
             WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(user_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    /// Restore a soft deleted user (admin only functionality)
    pub async fn restore(&self, id: &UserId) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "UPDATE users SET deleted_at = NULL, updated_at = NOW() 
             WHERE id = $1 AND deleted_at IS NOT NULL"
        )
        .bind(user_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }

    /// Get user including soft deleted ones (admin only)
    pub async fn get_with_deleted(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let row = sqlx::query(
            "SELECT id, firebase_uid, email, created_at, updated_at, deleted_at 
             FROM users WHERE id = $1"
        )
        .bind(user_uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let firebase_uid: String = row.try_get("firebase_uid")
                    .map_err(|e| RepoError::InvalidData(format!("Invalid firebase_uid: {}", e)))?;
                let email_str: String = row.get("email");
                let email = Email::new(email_str)
                    .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;

                let user_id: Uuid = row.get("id");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                let deleted_at: Option<chrono::DateTime<chrono::Utc>> = row.get("deleted_at");
                
                let user = User::from_existing_complete(
                    UserId::from_string(user_id.to_string()),
                    firebase_uid,
                    email,
                    Role::User,
                    crate::dom::values::Subscription::free(),
                    created_at,
                    updated_at,
                    deleted_at,
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    /// List soft deleted users (admin only)
    pub async fn list_deleted(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let rows = sqlx::query(
            "SELECT id, firebase_uid, email, created_at, updated_at, deleted_at 
             FROM users 
             WHERE deleted_at IS NOT NULL 
             ORDER BY deleted_at DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let firebase_uid: String = row.try_get("firebase_uid")
                .map_err(|e| RepoError::InvalidData(format!("Invalid firebase_uid: {}", e)))?;
            let email_str: String = row.get("email");
            let email = Email::new(email_str)
                .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;

            let user_id: Uuid = row.get("id");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let deleted_at: Option<chrono::DateTime<chrono::Utc>> = row.get("deleted_at");
            
            let user = User::from_existing_complete(
                UserId::from_string(user_id.to_string()),
                firebase_uid,
                email,
                Role::User,
                crate::dom::values::Subscription::free(),
                created_at,
                updated_at,
                deleted_at,
            );

            users.push(user);
        }

        Ok(users)
    }

    /// Count soft deleted users
    pub async fn count_deleted(&self) -> Result<u64, RepoError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM users WHERE deleted_at IS NOT NULL"
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    /// Permanently delete a soft deleted user (hard delete)
    pub async fn hard_delete(&self, id: &UserId) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        // Only allow hard delete of already soft deleted users
        let result = sqlx::query(
            "DELETE FROM users WHERE id = $1 AND deleted_at IS NOT NULL"
        )
        .bind(user_uuid)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }

        Ok(())
    }
}