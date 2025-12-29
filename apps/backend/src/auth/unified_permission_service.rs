// ============================================================================
// UNIFIED PERMISSION SERVICE - SINGLE SOURCE OF TRUTH
// ============================================================================
// This is the ONLY permission validation service in the system.
// All other permission systems have been removed.
//
// Architecture:
// - Database-backed (PostgreSQL via Diesel)
// - Redis cache with invalidation
// - Audit logging for all changes
// - Optimized single-query permission resolution
//
// Features:
// - Grant/revoke direct permissions
// - Assign/remove permission groups
// - Check permissions with wildcard support
// - Cache with automatic invalidation
// - Complete audit trail
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use diesel::prelude::*;
use std::sync::Arc;
// Note: wallet_group_memberships table not implemented yet
  // use crate::schemas::primary::{wallet_group_memberships};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::core::errors::AppError;
use crate::infrastructure::cache::unified_permission_cache::UnifiedPermissionCache;

// ============================================================================
// TYPES AND STRUCTURES
// ============================================================================

/// Detailed permission information with source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDetail {
    pub permission_string: String,
    pub permission_id: Uuid,
    pub source_type: PermissionSource,
    pub source_id: Uuid,
    pub source_name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_at: DateTime<Utc>,
    pub is_permanent: bool,
}

/// Permission source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionSource {
    Group,
    Direct,
}

/// Permission statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStats {
    pub total_permissions: i64,
    pub direct_permissions: i64,
    pub group_permissions: i64,
    pub permanent_permissions: i64,
    pub temporary_permissions: i64,
    pub groups_count: i64,
    pub expiring_soon_count: i64,
}

/// Grant permission request
#[derive(Debug, Clone)]
pub struct GrantPermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
    pub granted_by: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Revoke permission request
#[derive(Debug, Clone)]
pub struct RevokePermissionRequest {
    pub wallet_address: String,
    pub permission_string: String,
    pub revoked_by: String,
    pub reason: Option<String>,
}

/// Assign group request
#[derive(Debug, Clone)]
pub struct AssignGroupRequest {
    pub wallet_address: String,
    pub group_id: Uuid,
    pub assigned_by: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Remove group request
#[derive(Debug, Clone)]
pub struct RemoveGroupRequest {
    pub wallet_address: String,
    pub group_id: Uuid,
    pub removed_by: String,
    pub reason: Option<String>,
}

// ============================================================================
// UNIFIED PERMISSION SERVICE
// ============================================================================

/// The single source of truth for all permission operations
#[derive(Clone)]
pub struct UnifiedPermissionService {
    db_pool: &'static Pool<AsyncPgConnection>,
    cache: Option<Arc<UnifiedPermissionCache>>,
}

impl UnifiedPermissionService {
    /// Create new unified permission service with cache
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>, cache: Arc<UnifiedPermissionCache>) -> Self {
        Self { db_pool, cache: Some(cache) }
    }

    /// Create new unified permission service without cache (direct DB queries only)
    pub fn new_without_cache(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool, cache: None }
    }

    // ========================================================================
    // CORE PERMISSION CHECKING
    // ========================================================================

    /// Check if wallet has specific permission (supports wildcards)
    /// This is the primary permission validation method
    pub async fn has_permission(
        &self,
        wallet_address: &str,
        permission: &str,
    ) -> Result<bool, AppError> {
        let wallet_lower = wallet_address.to_lowercase();
        debug!("Checking permission '{}' for wallet: {}", permission, wallet_lower);

        // Try cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached_result) = cache.get_permission_check(&wallet_lower, permission).await {
                debug!("Cache hit for permission check: {}", permission);
                return Ok(cached_result);
            }
        }

        // Query database using optimized function
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct PermissionCheck {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
            wallet_has_permission: Option<bool>,
        }

        let has_permission = diesel::sql_query("SELECT wallet_has_permission($1, $2)")
            .bind::<diesel::sql_types::Text, _>(&wallet_lower)
            .bind::<diesel::sql_types::Text, _>(permission)
            .get_result::<PermissionCheck>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error checking permission: {}", e);
                AppError::database_error(format!("Failed to check permission: {}", e))
            })?;

        let result = has_permission.wallet_has_permission.unwrap_or(false);

        // Cache result
        if let Some(ref cache) = self.cache {
            cache.set_permission_check(&wallet_lower, permission, result).await;
        }

        Ok(result)
    }

    /// Get all permissions for a wallet with detailed information
    pub async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<PermissionDetail>, AppError> {
        let wallet_lower = wallet_address.to_lowercase();
        debug!("Fetching permissions for wallet: {}", wallet_lower);

        // Try cache first
        if let Some(ref cache) = self.cache {
            if let Some(cached_permissions) = cache.get_wallet_permissions(&wallet_lower).await {
                debug!("Cache hit for wallet permissions");
                return Ok(cached_permissions);
            }
        }

        // Query database using optimized function
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        // Define PermissionDetailRow inline since it's only used here
        #[derive(diesel::QueryableByName)]
        struct PermissionDetailRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub permission_string: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            pub permission_id: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub source_type: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            pub source_id: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            pub source_name: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            pub granted_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            pub is_permanent: bool,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT
                permission_string,
                permission_id::text,
                source_type,
                source_id::text,
                source_name,
                expires_at,
                granted_at,
                is_permanent
            FROM public.get_wallet_permissions_detailed_working($1)
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_lower)
        .load::<PermissionDetailRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Database error fetching wallet permissions: {}", e);
            AppError::database_error(format!("Failed to fetch permissions: {}", e))
        })?;

        let permissions: Vec<PermissionDetail> = rows
            .into_iter()
            .map(|row| {
                PermissionDetail {
                    permission_string: row.permission_string,
                    permission_id: row.permission_id
                        .and_then(|s| Uuid::parse_str(&s).ok())
                        .unwrap_or_else(Uuid::new_v4),
                    source_type: if row.source_type == "group" {
                        PermissionSource::Group
                    } else {
                        PermissionSource::Direct
                    },
                    source_id: row.source_id
                        .and_then(|s| Uuid::parse_str(&s).ok())
                        .unwrap_or_else(Uuid::new_v4),
                    source_name: row.source_name.unwrap_or_else(|| "Unknown".to_string()),
                    expires_at: row.expires_at,
                    granted_at: row.granted_at,
                    is_permanent: row.is_permanent,
                }
            })
            .collect();

        // Cache result
        if let Some(ref cache) = self.cache {
            cache.set_wallet_permissions(&wallet_lower, &permissions).await;
        }

        info!("Found {} permissions for wallet: {}", permissions.len(), wallet_lower);
        Ok(permissions)
    }

    /// Get permission strings only (for JWT generation)
    pub async fn get_permission_strings(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<String>, AppError> {
        let wallet_lower = wallet_address.to_lowercase();

        // Query database using optimized function
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct PermissionsJson {
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            get_wallet_effective_permissions: serde_json::Value,
        }

        let permissions_json = diesel::sql_query("SELECT get_wallet_effective_permissions($1)")
            .bind::<diesel::sql_types::Text, _>(&wallet_lower)
            .get_result::<PermissionsJson>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error fetching permission strings: {}", e);
                AppError::database_error(format!("Failed to fetch permission strings: {}", e))
            })?;

        let permission_strings: Vec<String> = if let serde_json::Value::Array(arr) = permissions_json.get_wallet_effective_permissions {
            arr.into_iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            vec![]
        };

        Ok(permission_strings)
    }

    /// Batch check multiple permissions at once
    pub async fn has_permissions_batch(
        &self,
        wallet_address: &str,
        permissions: &[String],
    ) -> Result<Vec<(String, bool)>, AppError> {
        let wallet_lower = wallet_address.to_lowercase();
        debug!("Batch checking {} permissions for wallet: {}", permissions.len(), wallet_lower);

        // Convert to PostgreSQL array
        let perms_array: Vec<String> = permissions.to_vec();

        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct BatchPermissionResult {
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_string: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            has_permission: bool,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT permission_string, has_permission
            FROM wallet_has_permissions_batch($1, $2)
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_lower)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(&perms_array)
        .load::<BatchPermissionResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Database error in batch permission check: {}", e);
            AppError::database_error(format!("Failed to batch check permissions: {}", e))
        })?;

        let results: Vec<(String, bool)> = rows
            .into_iter()
            .map(|row| (row.permission_string, row.has_permission))
            .collect();

        Ok(results)
    }

    // ========================================================================
    // DIRECT PERMISSION MANAGEMENT
    // ========================================================================

    /// Grant direct permission to wallet
    pub async fn grant_permission(
        &self,
        request: GrantPermissionRequest,
    ) -> Result<Uuid, AppError> {
        let wallet_lower = request.wallet_address.to_lowercase();
        info!(
            "Granting permission '{}' to wallet: {} by {}",
            request.permission_string, wallet_lower, request.granted_by
        );

        // Validate permission format
        Self::validate_permission_format(&request.permission_string)?;

        // Get or create permission
        let permission_id = self.get_or_create_permission(&request.permission_string).await?;

        // Insert into wallet_direct_permissions
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct InsertResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let direct_permission_id = diesel::sql_query(
            r#"
            INSERT INTO wallet_direct_permissions (
                wallet_address,
                permission_id,
                granted_at,
                expires_at,
                granted_by,
                grant_reason,
                is_active
            ) VALUES ($1, $2, NOW(), $3, $4, $5, TRUE)
            ON CONFLICT (wallet_address, permission_id)
            DO UPDATE SET
                is_active = TRUE,
                granted_at = NOW(),
                expires_at = EXCLUDED.expires_at,
                granted_by = EXCLUDED.granted_by,
                grant_reason = EXCLUDED.grant_reason
            RETURNING id
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_lower)
        .bind::<diesel::sql_types::Uuid, _>(permission_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(request.expires_at)
        .bind::<diesel::sql_types::Text, _>(&request.granted_by)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&request.reason)
        .get_result::<InsertResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Database error granting permission: {}", e);
            AppError::database_error(format!("Failed to grant permission: {}", e))
        })?
        .id;

        // Invalidate cache
        if let Some(ref cache) = self.cache {
            cache.invalidate_wallet(&wallet_lower).await;
        }

        // Audit log (trigger handles this automatically)

        info!("Successfully granted permission '{}' to wallet: {}", request.permission_string, wallet_lower);
        Ok(direct_permission_id)
    }

    /// Revoke direct permission from wallet
    pub async fn revoke_permission(
        &self,
        request: RevokePermissionRequest,
    ) -> Result<(), AppError> {
        let wallet_lower = request.wallet_address.to_lowercase();
        info!(
            "Revoking permission '{}' from wallet: {} by {}",
            request.permission_string, wallet_lower, request.revoked_by
        );

        // Get permission ID
        let permission_id = self.get_permission_id(&request.permission_string).await?
            .ok_or_else(|| AppError::not_found("Permission not found"))?;

        // Delete from wallet_direct_permissions
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        let rows_affected = diesel::sql_query(
            r#"
            DELETE FROM wallet_direct_permissions
            WHERE wallet_address = $1
              AND permission_id = $2
              AND is_active = TRUE
            "#
        )
        .bind::<diesel::sql_types::Text, _>(&wallet_lower)
        .bind::<diesel::sql_types::Uuid, _>(permission_id)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Database error revoking permission: {}", e);
            AppError::database_error(format!("Failed to revoke permission: {}", e))
        })? as u64;

        if rows_affected == 0 {
            warn!("Permission '{}' was not found for wallet: {}", request.permission_string, wallet_lower);
            return Err(AppError::not_found("Permission not found for this wallet"));
        }

        // Invalidate cache
        if let Some(ref cache) = self.cache {
            cache.invalidate_wallet(&wallet_lower).await;
        }

        // Audit log (trigger handles this automatically)

        info!("Successfully revoked permission '{}' from wallet: {}", request.permission_string, wallet_lower);
        Ok(())
    }

    // ========================================================================
    // GROUP MANAGEMENT
    // ========================================================================

    /// Assign wallet to permission group
    pub async fn assign_group(
        &self,
        request: AssignGroupRequest,
    ) -> Result<Uuid, AppError> {
        let wallet_lower = request.wallet_address.to_lowercase();
        info!(
            "Assigning group {} to wallet: {} by {}",
            request.group_id, wallet_lower, request.assigned_by
        );

        let mut conn = self.db_pool.get().await.map_err(|e| {
            AppError::database_error(format!("Failed to get database connection: {}", e))
        })?;

        // Use raw SQL for complex ON CONFLICT operation
        let query = format!(r#"
            INSERT INTO wallet_group_memberships (
                wallet_address,
                group_id,
                assigned_at,
                expires_at,
                assigned_by,
                assignment_reason,
                is_active
            ) VALUES ('{}', '{}', NOW(), '{}', '{}', '{}', TRUE)
            ON CONFLICT (wallet_address, group_id)
            DO UPDATE SET
                is_active = TRUE,
                assigned_at = NOW(),
                expires_at = EXCLUDED.expires_at,
                assigned_by = EXCLUDED.assigned_by,
                assignment_reason = EXCLUDED.assignment_reason
            RETURNING id
            "#,
            wallet_lower.replace("'", "''"),
            request.group_id,
            request.expires_at.map(|dt| dt.to_rfc3339()).unwrap_or("NULL".to_string()).replace("'", "''"),
            request.assigned_by.replace("'", "''"),
            request.reason.unwrap_or_default().replace("'", "''")
        );

        #[derive(diesel::QueryableByName)]
        struct AssignmentResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let assignment_id = diesel::sql_query(query)
            .get_result::<AssignmentResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error assigning group: {}", e);
                AppError::database_error(format!("Failed to assign group: {}", e))
            })?
            .id;

        // Invalidate cache
        if let Some(ref cache) = self.cache {
            cache.invalidate_wallet(&wallet_lower).await;
        }

        // Audit log (trigger handles this automatically)

        info!("Successfully assigned group {} to wallet: {}", request.group_id, wallet_lower);
        Ok(assignment_id)
    }

    /// Remove wallet from permission group
    pub async fn remove_group(
        &self,
        request: RemoveGroupRequest,
    ) -> Result<(), AppError> {
        let wallet_lower = request.wallet_address.to_lowercase();
        info!(
            "Removing group {} from wallet: {} by {}",
            request.group_id, wallet_lower, request.removed_by
        );

        // Get database connection
        let mut conn = self.db_pool.get().await.map_err(|e| {
            AppError::database_error(format!("Failed to get database connection: {}", e))
        })?;

        // Use raw SQL for DELETE operation
        let query = format!(r#"
            DELETE FROM wallet_group_assignments
            WHERE wallet_address = '{}'
              AND group_id = '{}'
              AND is_active = TRUE
            "#,
            wallet_lower.replace("'", "''"),
            request.group_id
        );

        let rows_affected = diesel::sql_query(query)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error removing group: {}", e);
                AppError::database_error(format!("Failed to remove group: {}", e))
            })?;

        if rows_affected == 0 {
            warn!("Group {} was not found for wallet: {}", request.group_id, wallet_lower);
            return Err(AppError::not_found("Group assignment not found for this wallet"));
        }

        // Invalidate cache
        if let Some(ref cache) = self.cache {
            cache.invalidate_wallet(&wallet_lower).await;
        }

        // Audit log (trigger handles this automatically)

        info!("Successfully removed group {} from wallet: {}", request.group_id, wallet_lower);
        Ok(())
    }

    // ========================================================================
    // STATISTICS AND UTILITIES
    // ========================================================================

    /// Get permission statistics for wallet
    pub async fn get_permission_stats(
        &self,
        wallet_address: &str,
    ) -> Result<PermissionStats, AppError> {
        let wallet_lower = wallet_address.to_lowercase();

        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct StatsRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_permissions: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            direct_permissions: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            group_permissions: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            permanent_permissions: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            temporary_permissions: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            groups_count: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            expiring_soon_count: i64,
        }

        let row = diesel::sql_query("SELECT * FROM get_wallet_permission_stats($1)")
            .bind::<diesel::sql_types::Text, _>(&wallet_lower)
            .get_result::<StatsRow>(&mut conn)
            .await
            .map_err(|e| {
                error!("Database error fetching permission stats: {}", e);
                AppError::database_error(format!("Failed to fetch permission stats: {}", e))
            })?;

        Ok(PermissionStats {
            total_permissions: row.total_permissions,
            direct_permissions: row.direct_permissions,
            group_permissions: row.group_permissions,
            permanent_permissions: row.permanent_permissions,
            temporary_permissions: row.temporary_permissions,
            groups_count: row.groups_count,
            expiring_soon_count: row.expiring_soon_count,
        })
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Validate permission format: platform:resource:action
    fn validate_permission_format(permission: &str) -> Result<(), AppError> {
        let parts: Vec<&str> = permission.split(':').collect();

        if parts.len() != 3 {
            return Err(AppError::validation_error(
                "Permission must be in format 'platform:resource:action'"
            ));
        }

        // Validate each part is not empty
        for part in &parts {
            if part.trim().is_empty() {
                return Err(AppError::validation_error(
                    "Permission parts cannot be empty"
                ));
            }
        }

        // Validate characters (alphanumeric, underscore, hyphen, asterisk only)
        let valid_chars = |s: &str| {
            s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '*')
        };

        for part in &parts {
            if !valid_chars(part) {
                return Err(AppError::validation_error(
                    "Permission parts can only contain alphanumeric characters, underscores, hyphens, and asterisks"
                ));
            }
        }

        Ok(())
    }

    /// Get or create permission in database
    async fn get_or_create_permission(&self, permission_string: &str) -> Result<Uuid, AppError> {
        let parts: Vec<&str> = permission_string.split(':').collect();

        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct PermissionId {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let permission_id = diesel::sql_query(
            r#"
            INSERT INTO permissions (
                permission_string,
                platform,
                resource,
                action,
                permission_type,
                is_active
            ) VALUES ($1, $2, $3, $4, 'manual', TRUE)
            ON CONFLICT (permission_string)
            DO UPDATE SET updated_at = NOW()
            RETURNING id
            "#
        )
        .bind::<diesel::sql_types::Text, _>(permission_string)
        .bind::<diesel::sql_types::Text, _>(parts[0])
        .bind::<diesel::sql_types::Text, _>(parts[1])
        .bind::<diesel::sql_types::Text, _>(parts[2])
        .get_result::<PermissionId>(&mut conn)
        .await
        .map_err(|e| {
            error!("Database error creating permission: {}", e);
            AppError::database_error(format!("Failed to create permission: {}", e))
        })?
        .id;

        Ok(permission_id)
    }

    /// Get permission ID by string
    async fn get_permission_id(&self, permission_string: &str) -> Result<Option<Uuid>, AppError> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| {
                error!("Pool error: {}", e);
                AppError::database_error(format!("Pool error: {}", e))
            })?;

        #[derive(QueryableByName)]
        struct PermissionIdResult {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let permission_id = diesel::sql_query(
            "SELECT id FROM permissions WHERE permission_string = $1 AND is_active = TRUE"
        )
        .bind::<diesel::sql_types::Text, _>(permission_string)
        .get_result::<PermissionIdResult>(&mut conn)
        .await
        .optional()
        .map_err(|e| {
            error!("Database error fetching permission ID: {}", e);
            AppError::database_error(format!("Failed to fetch permission ID: {}", e))
        })?
        .map(|r| r.id);

        Ok(permission_id)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_permission_format() {
        // Valid permissions
        assert!(UnifiedPermissionService::validate_permission_format("admin:users:read").is_ok());
        assert!(UnifiedPermissionService::validate_permission_format("epsx:analytics:view").is_ok());
        assert!(UnifiedPermissionService::validate_permission_format("admin:*:*").is_ok());

        // Invalid permissions
        assert!(UnifiedPermissionService::validate_permission_format("invalid").is_err());
        assert!(UnifiedPermissionService::validate_permission_format("admin:users").is_err());
        assert!(UnifiedPermissionService::validate_permission_format("admin::read").is_err());
        assert!(UnifiedPermissionService::validate_permission_format("admin:users:read:extra").is_err());
        assert!(UnifiedPermissionService::validate_permission_format("admin:users:read!").is_err());
    }
}
