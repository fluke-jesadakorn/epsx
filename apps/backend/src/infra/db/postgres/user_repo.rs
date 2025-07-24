// PostgreSQL User Repository Implementation

use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::app::ports::repositories::{UserRepo, RepoError};
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email, Role};
use super::DatabasePool;

pub struct PostgresUserRepo {
    pool: DatabasePool,
}

impl PostgresUserRepo {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    async fn get(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let row = sqlx::query(
            "SELECT id, email, role, subscription_tier, created_at, updated_at, is_active 
             FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let email_str: String = row.get("email");
                let email = Email::new(email_str)
                    .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
                
                let role_str: String = row.get("role");
                let role = match role_str.as_str() {
                    "free" => Role::Free,
                    "user" => Role::User,
                    "premium" => Role::Premium,
                    "moderator" => Role::Moderator,
                    "admin" => Role::Admin,
                    "super_admin" => Role::SuperAdmin,
                    _ => return Err(RepoError::InvalidData(format!("Invalid role: {}", role_str))),
                };

                let user_id: Uuid = row.get("id");
                let user = User::from_existing(
                    UserId::from_string(user_id.to_string()),
                    email,
                    role,
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&user.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        sqlx::query(
            "INSERT INTO users (id, email, role, subscription_tier, created_at, updated_at, is_active)
             VALUES ($1, $2, $3, $4, NOW(), NOW(), true)
             ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                role = EXCLUDED.role,
                subscription_tier = EXCLUDED.subscription_tier,
                updated_at = NOW()"
        )
        .bind(user_uuid)
        .bind(user.email().value())
        .bind(user.role().to_string())
        .bind(user.sub().tier.to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &UserId) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1"
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

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepoError> {
        let row = sqlx::query(
            "SELECT id, email, role, subscription_tier, created_at, updated_at, is_active 
             FROM users WHERE email = $1 AND is_active = true"
        )
        .bind(email.value())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let email_str: String = row.get("email");
                let email = Email::new(email_str)
                    .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
                
                let role_str: String = row.get("role");
                let role = match role_str.as_str() {
                    "free" => Role::Free,
                    "user" => Role::User,
                    "premium" => Role::Premium,
                    "moderator" => Role::Moderator,
                    "admin" => Role::Admin,
                    "super_admin" => Role::SuperAdmin,
                    _ => return Err(RepoError::InvalidData(format!("Invalid role: {}", role_str))),
                };

                let user_id: Uuid = row.get("id");
                let user = User::from_existing(
                    UserId::from_string(user_id.to_string()),
                    email,
                    role,
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    async fn find_by_role(&self, role: &Role) -> Result<Vec<User>, RepoError> {
        let role_str = role.to_string();
        
        let rows = sqlx::query(
            "SELECT id, email, role, subscription_tier, created_at, updated_at, is_active 
             FROM users WHERE role = $1 AND is_active = true 
             ORDER BY created_at DESC"
        )
        .bind(role_str)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let email_str: String = row.get("email");
            let email = Email::new(email_str)
                .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
            
            let role_str: String = row.get("role");
            let role = match role_str.as_str() {
                "free" => Role::Free,
                "user" => Role::User,
                "premium" => Role::Premium,
                "moderator" => Role::Moderator,
                "admin" => Role::Admin,
                "super_admin" => Role::SuperAdmin,
                _ => return Err(RepoError::InvalidData(format!("Invalid role: {}", role_str))),
            };

            let user_id: Uuid = row.get("id");
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                email,
                role,
            );

            users.push(user);
        }

        Ok(users)
    }

    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let rows = sqlx::query(
            "SELECT id, email, role, subscription_tier, created_at, updated_at, is_active 
             FROM users WHERE is_active = true 
             ORDER BY created_at DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let email_str: String = row.get("email");
            let email = Email::new(email_str)
                .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
            
            let role_str: String = row.get("role");
            let role = match role_str.as_str() {
                "free" => Role::Free,
                "user" => Role::User,
                "premium" => Role::Premium,
                "moderator" => Role::Moderator,
                "admin" => Role::Admin,
                "super_admin" => Role::SuperAdmin,
                _ => return Err(RepoError::InvalidData(format!("Invalid role: {}", role_str))),
            };

            let user_id: Uuid = row.get("id");
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                email,
                role,
            );

            users.push(user);
        }

        Ok(users)
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM users WHERE is_active = true"
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn save_batch(&self, users: &[User]) -> Result<(), RepoError> {
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        for user in users {
            let user_uuid = Uuid::parse_str(&user.id().to_string())
                .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
            
            sqlx::query(
                "INSERT INTO users (id, email, role, subscription_tier, created_at, updated_at, is_active)
                 VALUES ($1, $2, $3, $4, NOW(), NOW(), true)
                 ON CONFLICT (id) DO UPDATE SET
                    email = EXCLUDED.email,
                    role = EXCLUDED.role,
                    subscription_tier = EXCLUDED.subscription_tier,
                    updated_at = NOW()"
            )
            .bind(user_uuid)
            .bind(user.email().value())
            .bind(user.role().to_string())
            .bind(user.sub().tier.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<User>, RepoError> {
        let rows = sqlx::query(
            "SELECT id, email, role, subscription_tier, created_at, updated_at, is_active 
             FROM users WHERE is_active = true 
             ORDER BY created_at DESC"
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let email_str: String = row.get("email");
            let email = Email::new(email_str)
                .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
            
            let role_str: String = row.get("role");
            let role = match role_str.as_str() {
                "free" => Role::Free,
                "user" => Role::User,
                "premium" => Role::Premium,
                "moderator" => Role::Moderator,
                "admin" => Role::Admin,
                "super_admin" => Role::SuperAdmin,
                _ => return Err(RepoError::InvalidData(format!("Invalid role: {}", role_str))),
            };

            let user_id: Uuid = row.get("id");
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                email,
                role,
            );

            users.push(user);
        }

        Ok(users)
    }

    async fn find_by_id(&self, id: &UserId) -> Result<User, RepoError> {
        match self.get(id).await? {
            Some(user) => Ok(user),
            None => Err(RepoError::NotFound),
        }
    }
}