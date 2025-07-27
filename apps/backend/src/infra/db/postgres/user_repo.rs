// PostgreSQL User Repository Implementation

use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;

use crate::app::ports::repositories::{UserRepo, RepoError};
use crate::dom::entities::User;
use crate::dom::entities::iam::Permission;
use crate::dom::entities::permission_profile::{PermissionProfileId};
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
        let uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let row = sqlx::query(
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let fb_uid: String = row.try_get("firebase_uid")
                    .map_err(|e| RepoError::InvalidData(format!("Invalid firebase_uid: {}", e)))?;
                let email_str: String = row.get("email");
                let email = Email::new(email_str)
                    .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;

                let uid: Uuid = row.get("id");
                let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
                let role = Role::from_string(&role_str).unwrap_or(Role::User);
                
                let user = User::from_existing(
                    UserId::from_string(uid.to_string()),
                    fb_uid,
                    email,
                    role, // Load role from database
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> Result<(), RepoError> {
        let uuid = Uuid::parse_str(&user.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        sqlx::query(
            "INSERT INTO users (id, firebase_uid, email, role, created_at, updated_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())
             ON CONFLICT (id) DO UPDATE SET
                firebase_uid = EXCLUDED.firebase_uid,
                email = EXCLUDED.email,
                role = EXCLUDED.role,
                updated_at = NOW()"
        )
        .bind(uuid)
        .bind(user.firebase_uid())
        .bind(user.email().value())
        .bind(user.role().to_string())
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &UserId) -> Result<(), RepoError> {
        let uuid = Uuid::parse_str(&id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let result = sqlx::query(
            "DELETE FROM users WHERE id = $1"
        )
        .bind(uuid)
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
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users WHERE email = $1"
        )
        .bind(email.value())
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
                let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
                let role = Role::from_string(&role_str).unwrap_or(Role::User);
                
                let user = User::from_existing(
                    UserId::from_string(user_id.to_string()),
                    firebase_uid,
                    email,
                    role, // Load role from database
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    async fn find_by_firebase_uid(&self, firebase_uid: &str) -> Result<Option<User>, RepoError> {
        let row = sqlx::query(
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users WHERE firebase_uid = $1"
        )
        .bind(firebase_uid)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        match row {
            Some(row) => {
                let fb_uid: String = row.try_get("firebase_uid")
                    .map_err(|e| RepoError::InvalidData(format!("Invalid firebase_uid: {}", e)))?;
                let email_str: String = row.get("email");
                let email = Email::new(email_str)
                    .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;

                let uid: Uuid = row.get("id");
                let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
                let role = Role::from_string(&role_str).unwrap_or(Role::User);
                
                let user = User::from_existing(
                    UserId::from_string(uid.to_string()),
                    fb_uid,
                    email,
                    role, // Load role from database
                );

                Ok(Some(user))
            },
            None => Ok(None),
        }
    }

    async fn find_by_role(&self, _role: &Role) -> Result<Vec<User>, RepoError> {
        // Since roles are no longer stored in the database, return all users
        let rows = sqlx::query(
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users ORDER BY created_at DESC"
        )
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
            let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
            let role = Role::from_string(&role_str).unwrap_or(Role::User);
            
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                firebase_uid,
                email,
                role, // Load role from database
            );

            users.push(user);
        }

        Ok(users)
    }

    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let rows = sqlx::query(
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users ORDER BY created_at DESC 
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
            let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
            let role = Role::from_string(&role_str).unwrap_or(Role::User);
            
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                firebase_uid,
                email,
                role, // Load role from database
            );

            users.push(user);
        }

        Ok(users)
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM users"
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
            let uuid = Uuid::parse_str(&user.id().to_string())
                .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
            
            sqlx::query(
                "INSERT INTO users (id, firebase_uid, email, created_at, updated_at)
                 VALUES ($1, $2, $3, NOW(), NOW())
                 ON CONFLICT (id) DO UPDATE SET
                    firebase_uid = EXCLUDED.firebase_uid,
                    email = EXCLUDED.email,
                    updated_at = NOW()"
            )
            .bind(uuid)
            .bind(user.firebase_uid())
            .bind(user.email().value())
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
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users ORDER BY created_at DESC"
        )
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
            let role_str: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());
            let role = Role::from_string(&role_str).unwrap_or(Role::User);
            
            let user = User::from_existing(
                UserId::from_string(user_id.to_string()),
                firebase_uid,
                email,
                role, // Load role from database
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

    async fn find_users_for_auto_assignment(&self) -> Result<Vec<User>, RepoError> {
        // Find users who might be eligible for auto-assignment
        let rows = sqlx::query(
            "SELECT id, firebase_uid, email, role, created_at, updated_at 
             FROM users 
             WHERE created_at > NOW() - INTERVAL '30 days'
             ORDER BY created_at DESC
             LIMIT 1000"
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows {
            let uid: Uuid = row.get("id");
            let fb_uid: String = row.try_get("firebase_uid")
                    .map_err(|e| RepoError::InvalidData(format!("Invalid firebase_uid: {}", e)))?;
            let email_str: String = row.get("email");
            let email = Email::new(email_str)
                .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;

            let user = User::from_existing(
                UserId::from_string(uid.to_string()),
                fb_uid,
                email,
                Role::User,
            );
            users.push(user);
        }

        Ok(users)
    }

    async fn count_total_users(&self) -> Result<i64, RepoError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count)
    }

    async fn is_user_active_since(&self, user_id: &UserId, since: chrono::DateTime<chrono::Utc>) -> Result<bool, RepoError> {
        let uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        // Check if user has any activity since the given time
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM users 
             WHERE id = $1 AND updated_at > $2"
        )
        .bind(uuid)
        .bind(since)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    async fn has_good_payment_history(&self, _user_id: &UserId, _days: i64) -> Result<bool, RepoError> {
        // Placeholder implementation - would check payment history over the specified days
        Ok(true)
    }

    async fn health_check(&self) -> Result<(), RepoError> {
        sqlx::query("SELECT 1")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| RepoError::QueryError(format!("Health check failed: {}", e)))?;
        
        Ok(())
    }
}

// Additional methods for PostgresUserRepo
impl PostgresUserRepo {
    
    /// Get user permissions by resolving all assigned permission profiles
    pub async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<Permission>, RepoError> {
        let uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let rows = sqlx::query(
            "SELECT rp.permissions 
             FROM user_permission_assignments upa
             JOIN role_profiles rp ON upa.permission_profile_id = rp.id
             WHERE upa.user_id = $1 AND rp.is_active = true
             AND (upa.expires_at IS NULL OR upa.expires_at > NOW())"
        )
        .bind(uuid)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut permissions = Vec::new();
        for row in rows {
            let permissions_json: serde_json::Value = row.get("permissions");
            if let Some(perms_array) = permissions_json.as_array() {
                for perm in perms_array {
                    if let Some(perm_str) = perm.as_str() {
                        let parts: Vec<&str> = perm_str.splitn(2, '/').collect();
                        if parts.len() == 2 {
                            permissions.push(Permission::new(parts[0].to_string(), parts[1].to_string()));
                        } else {
                            permissions.push(Permission::new(perm_str.to_string(), "*".to_string()));
                        }
                    }
                }
            }
        }

        Ok(permissions)
    }
    
    /// Check if user has specific API access permission
    pub async fn resolve_api_access(&self, user_id: &UserId, api_path: &str, method: &str) -> Result<bool, RepoError> {
        let permissions = self.get_user_permissions(user_id).await?;
        
        // Check for exact match or wildcard match
        for permission in permissions {
            let resource = format!("{}:{}", api_path, method.to_lowercase());
            if permission.resource() == &resource || 
               permission.resource() == &format!("{}:*", api_path) ||
               permission.resource() == "*" {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check if user has route access permission
    pub async fn resolve_route_access(&self, user_id: &UserId, route: &str) -> Result<bool, RepoError> {
        let permissions = self.get_user_permissions(user_id).await?;
        
        // Check for route access permissions
        for permission in permissions {
            if permission.resource().starts_with("route:") {
                let route_pattern = permission.resource().strip_prefix("route:").unwrap_or("");
                if route_pattern == "*" || route == route_pattern || route.starts_with(route_pattern) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Get user permission assignments with metadata
    pub async fn get_user_permission_assignments(&self, user_id: &UserId) -> Result<Vec<(PermissionProfileId, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>)>, RepoError> {
        let user_uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        let rows = sqlx::query(
            "SELECT permission_profile_id, assigned_at, expires_at
             FROM user_permission_assignments
             WHERE user_id = $1
             ORDER BY assigned_at DESC"
        )
        .bind(user_uuid)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut assignments = Vec::new();
        for row in rows {
            let profile_id: Uuid = row.get("permission_profile_id");
            let assigned_at: chrono::DateTime<chrono::Utc> = row.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
            
            assignments.push((
                PermissionProfileId::from(profile_id),
                assigned_at,
                expires_at
            ));
        }

        Ok(assignments)
    }
    
    /// Assign permission profile to user
    pub async fn assign_permission_profile(
        &self, 
        user_id: &UserId, 
        profile_id: &PermissionProfileId,
        assigned_by: &UserId,
        expires_at: Option<chrono::DateTime<chrono::Utc>>
    ) -> Result<(), RepoError> {
        let user_uuid = Uuid::parse_str(&user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        let profile_uuid = Uuid::parse_str(profile_id.value())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;
        let assigned_by_uuid = Uuid::parse_str(&assigned_by.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UUID: {}", e)))?;

        sqlx::query(
            "INSERT INTO user_permission_assignments (user_id, permission_profile_id, assigned_by, assigned_at, expires_at)
             VALUES ($1, $2, $3, NOW(), $4)
             ON CONFLICT (user_id, permission_profile_id) DO UPDATE SET
                assigned_by = EXCLUDED.assigned_by,
                assigned_at = NOW(),
                expires_at = EXCLUDED.expires_at"
        )
        .bind(user_uuid)
        .bind(profile_uuid)
        .bind(assigned_by_uuid)
        .bind(expires_at)
        .execute(&*self.pool)
        .await
        .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }
}