use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Database-based role and permission management service
/// Stores roles/permissions in database instead of Firebase custom claims
#[derive(Clone)]
pub struct DatabaseRoleService {
    db_pool: PgPool,
}

/// User role and permission data
#[derive(Debug, Clone)]
pub struct UserRoleData {
    pub id: Uuid,
    pub firebase_uid: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub access_level: String,
    pub is_admin: bool,
    pub is_premium: bool,
    pub role_assigned_by: Option<String>,
    pub role_assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role assignment request
#[derive(Debug, Clone)]
pub struct RoleAssignmentRequest {
    pub role: String,
    pub assigned_by: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, Value>>,
}

/// Role assignment audit entry
#[derive(Debug, Clone)]
pub struct RoleAssignmentAudit {
    pub id: Uuid,
    pub firebase_uid: String,
    pub old_role: Option<String>,
    pub new_role: String,
    pub assigned_by: Option<String>,
    pub reason: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
}

/// Database role service trait
#[async_trait]
pub trait DatabaseRoleServiceTrait: Send + Sync {
    async fn get_user_role(&self, firebase_uid: &str) -> Result<Option<UserRoleData>, RoleServiceError>;
    async fn assign_role(&self, firebase_uid: &str, request: RoleAssignmentRequest) -> Result<UserRoleData, RoleServiceError>;
    async fn update_permissions(&self, firebase_uid: &str, permissions: Vec<String>) -> Result<(), RoleServiceError>;
    async fn revoke_role(&self, firebase_uid: &str, revoked_by: &str, reason: Option<String>) -> Result<(), RoleServiceError>;
    async fn list_users_by_role(&self, role: &str) -> Result<Vec<String>, RoleServiceError>;
    async fn get_role_assignment_history(&self, firebase_uid: &str) -> Result<Vec<RoleAssignmentAudit>, RoleServiceError>;
    async fn cleanup_expired_roles(&self) -> Result<u32, RoleServiceError>;
}

/// Role service errors
#[derive(Debug, thiserror::Error)]
pub enum RoleServiceError {
    #[error("User role not found: {0}")]
    RoleNotFound(String),
    
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    #[error("Role assignment failed: {0}")]
    AssignmentFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

impl DatabaseRoleService {
    /// Create new database role service
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl DatabaseRoleServiceTrait for DatabaseRoleService {
    /// Get user role and permissions from database
    async fn get_user_role(&self, firebase_uid: &str) -> Result<Option<UserRoleData>, RoleServiceError> {
        tracing::debug!("Getting role for Firebase user: {}", firebase_uid);
        
        let query = r#"
            SELECT id, firebase_uid, role, permissions, access_level, is_admin, is_premium,
                   role_assigned_by, role_assigned_at, expires_at, created_at, updated_at
            FROM user_roles_permissions
            WHERE firebase_uid = $1
        "#;
        
        let row = match sqlx::query(query)
            .bind(firebase_uid)
            .fetch_optional(&self.db_pool)
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => return Ok(None),
            Err(e) => {
                tracing::error!("Failed to get user role for {}: {}", firebase_uid, e);
                return Err(RoleServiceError::DatabaseError(e.to_string()));
            }
        };
        
        let permissions: Vec<String> = row.get::<Vec<String>, _>("permissions");
        
        let role_data = UserRoleData {
            id: row.get("id"),
            firebase_uid: row.get("firebase_uid"),
            role: row.get("role"),
            permissions,
            access_level: row.get("access_level"),
            is_admin: row.get("is_admin"),
            is_premium: row.get("is_premium"),
            role_assigned_by: row.get("role_assigned_by"),
            role_assigned_at: row.get("role_assigned_at"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        // Check if role is expired
        if let Some(expires_at) = role_data.expires_at {
            if Utc::now() > expires_at {
                tracing::warn!("Role expired for user {}: expired at {}", firebase_uid, expires_at);
                // Optionally revoke expired role here
            }
        }
        
        Ok(Some(role_data))
    }
    
    /// Assign role to user in database
    async fn assign_role(&self, firebase_uid: &str, request: RoleAssignmentRequest) -> Result<UserRoleData, RoleServiceError> {
        tracing::info!("Assigning role '{}' to Firebase user: {}", request.role, firebase_uid);
        
        // Validate role
        let (access_level, is_admin, permissions) = self.get_role_properties(&request.role)?;
        
        // Start transaction
        let mut tx = self.db_pool.begin().await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        // Get existing role for audit
        let existing_role = self.get_user_role(firebase_uid).await?;
        
        // Upsert role assignment
        let query = r#"
            INSERT INTO user_roles_permissions (
                firebase_uid, role, permissions, access_level, is_admin, is_premium,
                role_assigned_by, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (firebase_uid) DO UPDATE SET
                role = EXCLUDED.role,
                permissions = EXCLUDED.permissions,
                access_level = EXCLUDED.access_level,
                is_admin = EXCLUDED.is_admin,
                is_premium = EXCLUDED.is_premium,
                role_assigned_by = EXCLUDED.role_assigned_by,
                role_assigned_at = NOW(),
                expires_at = EXCLUDED.expires_at,
                updated_at = NOW()
            RETURNING id, firebase_uid, role, permissions, access_level, is_admin, is_premium,
                      role_assigned_by, role_assigned_at, expires_at, created_at, updated_at
        "#;
        
        let is_premium = request.role.contains("premium");
        
        let row = sqlx::query(query)
            .bind(firebase_uid)
            .bind(&request.role)
            .bind(&permissions)
            .bind(&access_level)
            .bind(is_admin)
            .bind(is_premium)
            .bind(&request.assigned_by)
            .bind(request.expires_at)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to assign role: {}", e);
                RoleServiceError::AssignmentFailed(e.to_string())
            })?;
        
        // Create audit log entry
        let audit_query = r#"
            INSERT INTO role_assignment_audit (
                firebase_uid, old_role, new_role, assigned_by, reason, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6)
        "#;
        
        let metadata_json = request.metadata
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .unwrap_or_default();
        
        sqlx::query(audit_query)
            .bind(firebase_uid)
            .bind(existing_role.as_ref().map(|r| &r.role))
            .bind(&request.role)
            .bind(&request.assigned_by)
            .bind(request.reason)
            .bind(metadata_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create audit log: {}", e);
                RoleServiceError::DatabaseError(e.to_string())
            })?;
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        let permissions: Vec<String> = row.get::<Vec<String>, _>("permissions");
        
        let role_data = UserRoleData {
            id: row.get("id"),
            firebase_uid: row.get("firebase_uid"),
            role: row.get("role"),
            permissions,
            access_level: row.get("access_level"),
            is_admin: row.get("is_admin"),
            is_premium: row.get("is_premium"),
            role_assigned_by: row.get("role_assigned_by"),
            role_assigned_at: row.get("role_assigned_at"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        tracing::info!("Successfully assigned role '{}' to user {}", request.role, firebase_uid);
        Ok(role_data)
    }
    
    /// Update user permissions directly
    async fn update_permissions(&self, firebase_uid: &str, permissions: Vec<String>) -> Result<(), RoleServiceError> {
        tracing::info!("Updating permissions for Firebase user: {}", firebase_uid);
        
        let query = r#"
            UPDATE user_roles_permissions
            SET permissions = $2, updated_at = NOW()
            WHERE firebase_uid = $1
        "#;
        
        let result = sqlx::query(query)
            .bind(firebase_uid)
            .bind(&permissions)
            .execute(&self.db_pool)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        if result.rows_affected() == 0 {
            return Err(RoleServiceError::RoleNotFound(firebase_uid.to_string()));
        }
        
        tracing::info!("Successfully updated permissions for user {}", firebase_uid);
        Ok(())
    }
    
    /// Revoke user role
    async fn revoke_role(&self, firebase_uid: &str, revoked_by: &str, reason: Option<String>) -> Result<(), RoleServiceError> {
        tracing::info!("Revoking role for Firebase user: {}", firebase_uid);
        
        // Get existing role for audit
        let existing_role = self.get_user_role(firebase_uid).await?
            .ok_or_else(|| RoleServiceError::RoleNotFound(firebase_uid.to_string()))?;
        
        // Start transaction
        let mut tx = self.db_pool.begin().await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        // Reset to basic user role
        let basic_role = "user-basic-001";
        let (access_level, is_admin, permissions) = self.get_role_properties(basic_role)?;
        
        let update_query = r#"
            UPDATE user_roles_permissions
            SET role = $2, permissions = $3, access_level = $4, is_admin = $5, is_premium = FALSE,
                role_assigned_by = $6, role_assigned_at = NOW(), expires_at = NULL, updated_at = NOW()
            WHERE firebase_uid = $1
        "#;
        
        sqlx::query(update_query)
            .bind(firebase_uid)
            .bind(basic_role)
            .bind(&permissions)
            .bind(&access_level)
            .bind(is_admin)
            .bind(revoked_by)
            .execute(&mut *tx)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        // Create audit log entry
        let audit_query = r#"
            INSERT INTO role_assignment_audit (
                firebase_uid, old_role, new_role, assigned_by, reason, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6)
        "#;
        
        let metadata = serde_json::json!({
            "action": "role_revoked",
            "previous_role": existing_role.role
        });
        
        sqlx::query(audit_query)
            .bind(firebase_uid)
            .bind(&existing_role.role)
            .bind(basic_role)
            .bind(revoked_by)
            .bind(reason)
            .bind(metadata)
            .execute(&mut *tx)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        tracing::info!("Successfully revoked role for user {}", firebase_uid);
        Ok(())
    }
    
    /// List all Firebase UIDs with specific role
    async fn list_users_by_role(&self, role: &str) -> Result<Vec<String>, RoleServiceError> {
        tracing::debug!("Listing users with role: {}", role);
        
        let query = r#"
            SELECT firebase_uid
            FROM user_roles_permissions
            WHERE role = $1 
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY role_assigned_at DESC
        "#;
        
        let rows = sqlx::query(query)
            .bind(role)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        let firebase_uids = rows.into_iter()
            .map(|row| row.get::<String, _>("firebase_uid"))
            .collect();
        
        Ok(firebase_uids)
    }
    
    /// Get role assignment history for user
    async fn get_role_assignment_history(&self, firebase_uid: &str) -> Result<Vec<RoleAssignmentAudit>, RoleServiceError> {
        tracing::debug!("Getting role assignment history for: {}", firebase_uid);
        
        let query = r#"
            SELECT id, firebase_uid, old_role, new_role, assigned_by, reason, metadata, timestamp
            FROM role_assignment_audit
            WHERE firebase_uid = $1
            ORDER BY timestamp DESC
            LIMIT 50
        "#;
        
        let rows = sqlx::query(query)
            .bind(firebase_uid)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        let audit_entries = rows.into_iter().map(|row| {
            let metadata: Value = row.get("metadata");
            let metadata_map: HashMap<String, Value> = serde_json::from_value(metadata)
                .unwrap_or_default();
            
            RoleAssignmentAudit {
                id: row.get("id"),
                firebase_uid: row.get("firebase_uid"),
                old_role: row.get("old_role"),
                new_role: row.get("new_role"),
                assigned_by: row.get("assigned_by"),
                reason: row.get("reason"),
                metadata: metadata_map,
                timestamp: row.get("timestamp"),
            }
        }).collect();
        
        Ok(audit_entries)
    }
    
    /// Clean up expired roles
    async fn cleanup_expired_roles(&self) -> Result<u32, RoleServiceError> {
        tracing::info!("Cleaning up expired roles");
        
        // Find expired roles
        let select_query = r#"
            SELECT firebase_uid, role
            FROM user_roles_permissions
            WHERE expires_at IS NOT NULL AND expires_at < NOW()
        "#;
        
        let expired_roles = sqlx::query(select_query)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| RoleServiceError::DatabaseError(e.to_string()))?;
        
        let mut cleaned_count = 0;
        
        for row in expired_roles {
            let firebase_uid: String = row.get("firebase_uid");
            let old_role: String = row.get("role");
            
            // Reset to basic role
            if let Err(e) = self.revoke_role(&firebase_uid, "system", Some("Role expired".to_string())).await {
                tracing::error!("Failed to revoke expired role for {}: {}", firebase_uid, e);
            } else {
                cleaned_count += 1;
                tracing::info!("Revoked expired role '{}' for user {}", old_role, firebase_uid);
            }
        }
        
        tracing::info!("Successfully cleaned up {} expired roles", cleaned_count);
        Ok(cleaned_count)
    }
}

impl DatabaseRoleService {
    /// Get role properties (access level, admin status, permissions)
    fn get_role_properties(&self, role: &str) -> Result<(String, bool, Vec<String>), RoleServiceError> {
        let (access_level, is_admin, permissions) = match role {
            "super_admin" => (
                "super".to_string(),
                true,
                vec![
                    "admin:*".to_string(),
                    "create:users".to_string(),
                    "delete:users".to_string(),
                    "manage:roles".to_string(),
                    "read:analytics".to_string(),
                    "manage:system".to_string(),
                    "read:profile".to_string(),
                    "update:profile".to_string(),
                ]
            ),
            "admin-full-004" | "admin" => (
                "full".to_string(),
                true,
                vec![
                    "admin:users".to_string(),
                    "create:users".to_string(),
                    "update:users".to_string(),
                    "read:analytics".to_string(),
                    "manage:profiles".to_string(),
                    "read:profile".to_string(),
                    "update:profile".to_string(),
                ]
            ),
            "moderator-standard-003" | "moderator" => (
                "standard".to_string(),
                true,
                vec![
                    "admin:limited".to_string(),
                    "read:users".to_string(),
                    "update:users".to_string(),
                    "read:analytics".to_string(),
                    "read:profile".to_string(),
                    "update:profile".to_string(),
                ]
            ),
            "user-premium-002" | "premium" => (
                "none".to_string(),
                false,
                vec![
                    "read:premium_analytics".to_string(),
                    "create:alerts".to_string(),
                    "read:profile".to_string(),
                    "update:profile".to_string(),
                ]
            ),
            "user-basic-001" | "user" => (
                "none".to_string(),
                false,
                vec![
                    "read:profile".to_string(),
                    "update:profile".to_string(),
                ]
            ),
            _ => {
                return Err(RoleServiceError::InvalidRole(format!("Unknown role: {}", role)));
            }
        };
        
        Ok((access_level, is_admin, permissions))
    }
}