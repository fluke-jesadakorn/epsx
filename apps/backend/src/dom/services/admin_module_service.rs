use crate::core::errors::AppError;
use crate::core::permission_constants::{AdminModuleValidator, get_all_admin_module_codes};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAdminModule {
    pub id: String,
    pub firebase_uid: String,
    pub module_code: String,
    pub granted_by: Option<String>,
    pub granted_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModule {
    pub id: String,
    pub module_code: String,
    pub module_name: String,
    pub description: String,
    pub category: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub requires_modules: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAssignmentRequest {
    pub firebase_uid: String,
    pub module_codes: Vec<String>,
    pub granted_by: String,
    pub granted_reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminModulePermissions {
    pub module_code: String,
    pub api_endpoints: Vec<String>,
    pub frontend_routes: Vec<String>,
    pub permissions: Vec<String>,
    pub resource_patterns: Vec<String>,
    pub access_level: String,
}

pub struct AdminModuleService {
    pool: PgPool,
}

impl AdminModuleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all available admin modules
    pub async fn get_all_admin_modules(&self) -> Result<Vec<AdminModule>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, module_code, module_name, description, category, 
                   icon, color, sort_order, is_active, requires_modules,
                   created_at, updated_at
            FROM admin_modules 
            WHERE is_active = true
            ORDER BY COALESCE(sort_order, 0) ASC, module_name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch admin modules: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        let modules: Vec<AdminModule> = rows.into_iter().map(|row| AdminModule {
            id: row.id.to_string(),
            module_code: row.module_code,
            module_name: row.module_name,
            description: row.description,
            category: row.category,
            icon: row.icon,
            color: row.color,
            sort_order: row.sort_order.unwrap_or(0),
            is_active: row.is_active.unwrap_or(true),
            requires_modules: row.requires_modules.unwrap_or_default(),
            created_at: row.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| Utc::now()),
        }).collect();

        info!("Retrieved {} admin modules", modules.len());
        Ok(modules)
    }

    /// Get user's assigned admin modules
    pub async fn get_user_admin_modules(&self, firebase_uid: &str) -> Result<Vec<String>, AppError> {
        let module_codes: Vec<String> = sqlx::query!(
            r#"
            SELECT DISTINCT uar.module_code
            FROM user_admin_roles uar
            JOIN admin_modules am ON uar.module_code = am.module_code
            WHERE uar.firebase_uid = $1
              AND uar.is_active = true
              AND am.is_active = true
              AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
            ORDER BY uar.module_code
            "#,
            firebase_uid
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user admin modules for {}: {}", firebase_uid, e);
            AppError::DatabaseError(e.to_string())
        })?
        .into_iter()
        .map(|row| row.module_code)
        .collect();

        info!("User {} has {} admin modules: {:?}", firebase_uid, module_codes.len(), module_codes);
        Ok(module_codes)
    }

    /// Get detailed user admin module assignments
    pub async fn get_user_admin_module_details(&self, firebase_uid: &str) -> Result<Vec<UserAdminModule>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT uar.id, uar.firebase_uid, uar.module_code, uar.granted_by,
                   uar.granted_reason, uar.expires_at, uar.is_active,
                   uar.assignment_metadata, uar.created_at, uar.updated_at
            FROM user_admin_roles uar
            JOIN admin_modules am ON uar.module_code = am.module_code
            WHERE uar.firebase_uid = $1
              AND am.is_active = true
            ORDER BY uar.created_at DESC
            "#,
            firebase_uid
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch user admin module details for {}: {}", firebase_uid, e);
            AppError::DatabaseError(e.to_string())
        })?;

        let assignments: Vec<UserAdminModule> = rows.into_iter().map(|row| UserAdminModule {
            id: row.id.to_string(),
            firebase_uid: row.firebase_uid,
            module_code: row.module_code,
            granted_by: row.granted_by,
            granted_reason: row.granted_reason,
            expires_at: row.expires_at,
            is_active: row.is_active.unwrap_or(true),
            assignment_metadata: row.assignment_metadata.unwrap_or_else(|| serde_json::json!({})),
            created_at: row.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: row.updated_at.unwrap_or_else(|| Utc::now()),
        }).collect();

        Ok(assignments)
    }

    /// Check if user has specific admin module access
    pub async fn user_has_admin_module(&self, firebase_uid: &str, module_code: &str) -> Result<bool, AppError> {
        let has_access = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_admin_roles uar
                JOIN admin_modules am ON uar.module_code = am.module_code
                WHERE uar.firebase_uid = $1
                  AND uar.module_code = $2
                  AND uar.is_active = true
                  AND am.is_active = true
                  AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
            ) as has_access
            "#,
            firebase_uid,
            module_code
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to check admin module access for {} -> {}: {}", firebase_uid, module_code, e);
            AppError::DatabaseError(e.to_string())
        })?
        .has_access.unwrap_or(false);

        Ok(has_access)
    }

    /// Check if user has any admin modules (is an admin)
    pub async fn user_is_admin(&self, firebase_uid: &str) -> Result<bool, AppError> {
        let is_admin = sqlx::query!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_admin_roles uar
                JOIN admin_modules am ON uar.module_code = am.module_code
                WHERE uar.firebase_uid = $1
                  AND uar.is_active = true
                  AND am.is_active = true
                  AND (uar.expires_at IS NULL OR uar.expires_at > NOW())
            ) as is_admin
            "#,
            firebase_uid
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to check admin status for {}: {}", firebase_uid, e);
            AppError::DatabaseError(e.to_string())
        })?
        .is_admin.unwrap_or(false);

        Ok(is_admin)
    }

    /// Assign admin modules to a user
    pub async fn assign_admin_modules(&self, request: &ModuleAssignmentRequest) -> Result<Vec<String>, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to start transaction: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        let mut assigned_modules = Vec::new();

        for module_code in &request.module_codes {
            // Check if module exists and is active
            let module_exists = sqlx::query!(
                "SELECT EXISTS(SELECT 1 FROM admin_modules WHERE module_code = $1 AND is_active = true) as exists",
                module_code
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to check module existence for {}: {}", module_code, e);
                AppError::DatabaseError(e.to_string())
            })?
            .exists.unwrap_or(false);

            if !module_exists {
                warn!("Attempted to assign non-existent module: {}", module_code);
                continue;
            }

            // Insert or update assignment (upsert)
            let assignment_id = sqlx::query!(
                r#"
                INSERT INTO user_admin_roles (firebase_uid, module_code, granted_by, granted_reason, expires_at, is_active, assignment_metadata)
                VALUES ($1, $2, $3, $4, $5, true, '{}')
                ON CONFLICT (firebase_uid, module_code) 
                DO UPDATE SET 
                    granted_by = EXCLUDED.granted_by,
                    granted_reason = EXCLUDED.granted_reason,
                    expires_at = EXCLUDED.expires_at,
                    is_active = true,
                    updated_at = NOW()
                RETURNING id
                "#,
                request.firebase_uid,
                module_code,
                request.granted_by,
                request.granted_reason,
                request.expires_at
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to assign module {} to user {}: {}", module_code, request.firebase_uid, e);
                AppError::DatabaseError(e.to_string())
            })?;

            // Log assignment in audit trail
            sqlx::query!(
                r#"
                INSERT INTO admin_role_audit (firebase_uid, module_code, action, new_status, performed_by, reason, timestamp)
                VALUES ($1, $2, 'granted', $3, $4, $5, NOW())
                "#,
                request.firebase_uid,
                module_code,
                serde_json::json!({
                    "assignment_id": assignment_id.id,
                    "expires_at": request.expires_at,
                    "is_active": true
                }),
                request.granted_by,
                request.granted_reason
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to log admin role audit for {}: {}", module_code, e);
                AppError::DatabaseError(e.to_string())
            })?;

            assigned_modules.push(module_code.clone());
            info!("Assigned module {} to user {} by {}", module_code, request.firebase_uid, request.granted_by);
        }

        tx.commit().await.map_err(|e| {
            error!("Failed to commit admin module assignment transaction: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        info!("Successfully assigned {} modules to user {}", assigned_modules.len(), request.firebase_uid);
        Ok(assigned_modules)
    }

    /// Revoke admin modules from a user
    pub async fn revoke_admin_modules(&self, firebase_uid: &str, module_codes: Vec<String>, revoked_by: &str, reason: &str) -> Result<Vec<String>, AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            error!("Failed to start transaction: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        let mut revoked_modules = Vec::new();

        for module_code in &module_codes {
            // Deactivate assignment
            let affected_rows = sqlx::query!(
                r#"
                UPDATE user_admin_roles 
                SET is_active = false, updated_at = NOW()
                WHERE firebase_uid = $1 AND module_code = $2 AND is_active = true
                "#,
                firebase_uid,
                module_code
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to revoke module {} from user {}: {}", module_code, firebase_uid, e);
                AppError::DatabaseError(e.to_string())
            })?;

            if affected_rows.rows_affected() > 0 {
                // Log revocation in audit trail
                sqlx::query!(
                    r#"
                    INSERT INTO admin_role_audit (firebase_uid, module_code, action, new_status, performed_by, reason, timestamp)
                    VALUES ($1, $2, 'revoked', $3, $4, $5, NOW())
                    "#,
                    firebase_uid,
                    module_code,
                    serde_json::json!({
                        "is_active": false,
                        "revoked_at": Utc::now()
                    }),
                    revoked_by,
                    reason
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to log admin role revocation audit for {}: {}", module_code, e);
                    AppError::DatabaseError(e.to_string())
                })?;

                revoked_modules.push(module_code.clone());
                info!("Revoked module {} from user {} by {}", module_code, firebase_uid, revoked_by);
            }
        }

        tx.commit().await.map_err(|e| {
            error!("Failed to commit admin module revocation transaction: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

        info!("Successfully revoked {} modules from user {}", revoked_modules.len(), firebase_uid);
        Ok(revoked_modules)
    }

    /// Assign ALL admin modules to a user (for jesadakorn.kirtnu@gmail.com)
    pub async fn assign_all_admin_modules(&self, firebase_uid: &str, granted_by: &str, reason: &str) -> Result<Vec<String>, AppError> {
        let all_modules = get_all_admin_module_codes();
        
        let request = ModuleAssignmentRequest {
            firebase_uid: firebase_uid.to_string(),
            module_codes: all_modules,
            granted_by: granted_by.to_string(),
            granted_reason: reason.to_string(),
            expires_at: None, // No expiration for full admin
        };

        info!("Assigning ALL {} admin modules to user {} by {}", request.module_codes.len(), firebase_uid, granted_by);
        self.assign_admin_modules(&request).await
    }

    /// Get module permissions for API/route validation
    pub async fn get_module_permissions(&self, module_code: &str) -> Result<Option<AdminModulePermissions>, AppError> {
        let permissions = sqlx::query!(
            r#"
            SELECT module_code, api_endpoints, frontend_routes, permissions, 
                   resource_patterns, access_level, description
            FROM admin_module_permissions 
            WHERE module_code = $1
            "#,
            module_code
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch permissions for module {}: {}", module_code, e);
            AppError::DatabaseError(e.to_string())
        })?;

        if let Some(row) = permissions {
            Ok(Some(AdminModulePermissions {
                module_code: row.module_code,
                api_endpoints: row.api_endpoints,
                frontend_routes: row.frontend_routes,
                permissions: row.permissions,
                resource_patterns: row.resource_patterns,
                access_level: row.access_level.unwrap_or_else(|| "standard".to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Validate endpoint access for user modules
    pub async fn can_access_endpoint(&self, firebase_uid: &str, endpoint: &str) -> Result<bool, AppError> {
        let user_modules = self.get_user_admin_modules(firebase_uid).await?;
        Ok(AdminModuleValidator::can_access_endpoint(&user_modules, endpoint))
    }

    /// Validate frontend route access for user modules
    pub async fn can_access_route(&self, firebase_uid: &str, route: &str) -> Result<bool, AppError> {
        let user_modules = self.get_user_admin_modules(firebase_uid).await?;
        Ok(AdminModuleValidator::can_access_route(&user_modules, route))
    }

    /// Get audit trail for admin role changes
    pub async fn get_admin_role_audit(&self, firebase_uid: &str, limit: Option<i32>) -> Result<Vec<serde_json::Value>, AppError> {
        let limit = limit.unwrap_or(50).min(500); // Max 500 records

        let audit_records = sqlx::query!(
            r#"
            SELECT firebase_uid, module_code, action, old_status, new_status, 
                   performed_by, reason, metadata, timestamp
            FROM admin_role_audit
            WHERE firebase_uid = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            firebase_uid,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch admin role audit for {}: {}", firebase_uid, e);
            AppError::DatabaseError(e.to_string())
        })?;

        let audit_json: Vec<serde_json::Value> = audit_records
            .into_iter()
            .map(|record| serde_json::json!({
                "firebase_uid": record.firebase_uid,
                "module_code": record.module_code,
                "action": record.action,
                "old_status": record.old_status,
                "new_status": record.new_status,
                "performed_by": record.performed_by,
                "reason": record.reason,
                "metadata": record.metadata,
                "timestamp": record.timestamp
            }))
            .collect();

        Ok(audit_json)
    }
}