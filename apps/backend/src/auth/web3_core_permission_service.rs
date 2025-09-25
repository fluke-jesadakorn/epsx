// Web3 Core Permission Management Service
// Handles basic permission operations: grant, revoke, check, list

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::web3_shared_types::{
    PermissionInfo, PermissionVerificationResult, Web3PermissionError, Web3PermissionResult
};

/// Core Web3 permission management service
#[derive(Clone)]
pub struct Web3CorePermissionService {
    db_pool: PgPool,
    cache_duration_minutes: i64,
}

impl Web3CorePermissionService {
    /// Create new core permission service
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            cache_duration_minutes: 30, // 30 minute cache
        }
    }

    /// Get all permissions for a wallet address
    pub async fn get_user_permissions(&self, wallet_address: &str) -> Web3PermissionResult<Vec<PermissionInfo>> {
        info!("🔍 Fetching all permissions for wallet: {}", wallet_address);
        
        let lowercase_wallet = wallet_address.to_lowercase();
        
        // Query database for all active permissions
        let permission_rows = sqlx::query!(
            r#"
            SELECT 
                permission,
                'manual' as permission_type,
                true as is_active,
                expires_at,
                granted_at,
                last_verified_at,
                verification_data
            FROM user_permissions up
            JOIN users u ON up.user_id = u.id
            WHERE LOWER(u.wallet_address) = $1 
            AND up.is_active = true
            AND (up.expires_at IS NULL OR up.expires_at > NOW())
            ORDER BY up.granted_at DESC
            "#,
            lowercase_wallet
        )
        .fetch_all(&self.db_pool)
        .await?;

        let permissions = permission_rows
            .into_iter()
            .map(|row| PermissionInfo {
                permission: row.permission,
                permission_type: row.permission_type,
                is_active: row.is_active,
                expires_at: row.expires_at,
                granted_at: row.granted_at,
                last_verified_at: row.last_verified_at,
                verification_data: row.verification_data,
            })
            .collect();

        info!("✅ Found {} permissions for wallet: {}", permissions.len(), wallet_address);
        Ok(permissions)
    }

    /// Check if wallet has specific permission
    pub async fn has_permission(&self, wallet_address: &str, permission: &str) -> Web3PermissionResult<bool> {
        debug!("🔎 Checking permission '{}' for wallet: {}", permission, wallet_address);
        
        let lowercase_wallet = wallet_address.to_lowercase();
        
        // Check direct permissions
        let has_direct = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user_permissions up
            JOIN users u ON up.user_id = u.id
            WHERE LOWER(u.wallet_address) = $1 
            AND up.permission = $2
            AND up.is_active = true
            AND (up.expires_at IS NULL OR up.expires_at > NOW())
            "#,
            lowercase_wallet,
            permission
        )
        .fetch_one(&self.db_pool)
        .await?
        .count
        .unwrap_or(0) > 0;

        if has_direct {
            debug!("✅ Direct permission found for wallet: {}", wallet_address);
            return Ok(true);
        }

        // Check wildcard permissions (admin:*:*, epsx:*:*)
        let permission_parts: Vec<&str> = permission.split(':').collect();
        let wildcard_patterns = if permission_parts.len() >= 2 {
            vec![
                format!("{}:*:*", permission_parts[0]),
                format!("{}:{}:*", permission_parts[0], permission_parts.get(1).unwrap_or(&"")),
            ]
        } else {
            vec![]
        };

        for pattern in wildcard_patterns {
            let has_wildcard = sqlx::query!(
                r#"
                SELECT COUNT(*) as count
                FROM user_permissions up
                JOIN users u ON up.user_id = u.id
                WHERE LOWER(u.wallet_address) = $1 
                AND up.permission = $2
                AND up.is_active = true
                AND (up.expires_at IS NULL OR up.expires_at > NOW())
                "#,
                lowercase_wallet,
                pattern
            )
            .fetch_one(&self.db_pool)
            .await?
            .count
            .unwrap_or(0) > 0;

            if has_wildcard {
                debug!("✅ Wildcard permission '{}' found for wallet: {}", pattern, wallet_address);
                return Ok(true);
            }
        }

        debug!("❌ No permission found for wallet: {}", wallet_address);
        Ok(false)
    }

    /// Grant manual permission to wallet
    pub async fn grant_manual_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        expires_at: Option<DateTime<Utc>>,
        granted_by: Option<Uuid>,
    ) -> Web3PermissionResult<()> {
        info!("✅ Granting manual permission '{}' to wallet: {}", permission, wallet_address);
        
        let lowercase_wallet = wallet_address.to_lowercase();
        
        // Find or create user
        let user_id = self.find_or_create_user(&lowercase_wallet).await?;
        
        // Insert permission
        sqlx::query!(
            r#"
            INSERT INTO user_permissions (user_id, permission, expires_at, granted_by, is_active)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (user_id, permission) 
            DO UPDATE SET 
                expires_at = $3,
                granted_by = $4,
                is_active = true,
                granted_at = NOW()
            "#,
            user_id,
            permission,
            expires_at,
            granted_by
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Successfully granted permission '{}' to wallet: {}", permission, wallet_address);
        Ok(())
    }

    /// Revoke permission from wallet
    pub async fn revoke_permission(
        &self,
        wallet_address: &str,
        permission: &str,
        revoked_by: Option<Uuid>,
    ) -> Web3PermissionResult<()> {
        info!("❌ Revoking permission '{}' from wallet: {}", permission, wallet_address);
        
        let lowercase_wallet = wallet_address.to_lowercase();
        
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user_permissions 
            SET is_active = false, revoked_by = $3, revoked_at = NOW()
            FROM users u
            WHERE user_permissions.user_id = u.id
            AND LOWER(u.wallet_address) = $1
            AND user_permissions.permission = $2
            AND user_permissions.is_active = true
            "#,
            lowercase_wallet,
            permission,
            revoked_by
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            info!("✅ Successfully revoked permission '{}' from wallet: {}", permission, wallet_address);
        } else {
            warn!("⚠️ No active permission found to revoke for wallet: {}", wallet_address);
        }

        Ok(())
    }

    /// Verify multiple permissions for wallet (batch operation)
    pub async fn verify_permissions_batch(
        &self,
        wallet_address: &str,
        permissions: &[String],
    ) -> Web3PermissionResult<HashMap<String, bool>> {
        debug!("🔎 Batch verifying {} permissions for wallet: {}", permissions.len(), wallet_address);
        
        let mut results = HashMap::new();
        
        // Use batch query for efficiency
        for permission in permissions {
            let has_perm = self.has_permission(wallet_address, permission).await?;
            results.insert(permission.clone(), has_perm);
        }
        
        debug!("✅ Batch verification complete for wallet: {}", wallet_address);
        Ok(results)
    }

    /// Get permission statistics for wallet
    pub async fn get_permission_stats(&self, wallet_address: &str) -> Web3PermissionResult<PermissionStats> {
        let lowercase_wallet = wallet_address.to_lowercase();
        
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_permissions,
                COUNT(*) FILTER (WHERE expires_at IS NULL) as permanent_permissions,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at > NOW()) as temporary_permissions,
                COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at <= NOW()) as expired_permissions
            FROM user_permissions up
            JOIN users u ON up.user_id = u.id
            WHERE LOWER(u.wallet_address) = $1 AND up.is_active = true
            "#,
            lowercase_wallet
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(PermissionStats {
            total_permissions: stats.total_permissions.unwrap_or(0) as u32,
            permanent_permissions: stats.permanent_permissions.unwrap_or(0) as u32,
            temporary_permissions: stats.temporary_permissions.unwrap_or(0) as u32,
            expired_permissions: stats.expired_permissions.unwrap_or(0) as u32,
        })
    }

    /// Helper: Find or create user by wallet address
    async fn find_or_create_user(&self, wallet_address: &str) -> Web3PermissionResult<Uuid> {
        // Try to find existing user
        if let Some(row) = sqlx::query!(
            "SELECT id FROM users WHERE LOWER(wallet_address) = $1",
            wallet_address.to_lowercase()
        )
        .fetch_optional(&self.db_pool)
        .await?
        {
            return Ok(row.id);
        }

        // Create new user
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO users (id, wallet_address, created_at)
            VALUES ($1, $2, NOW())
            "#,
            user_id,
            wallet_address
        )
        .execute(&self.db_pool)
        .await?;

        info!("👤 Created new user for wallet: {}", wallet_address);
        Ok(user_id)
    }
}

/// Permission statistics for a wallet
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionStats {
    pub total_permissions: u32,
    pub permanent_permissions: u32,
    pub temporary_permissions: u32,
    pub expired_permissions: u32,
}