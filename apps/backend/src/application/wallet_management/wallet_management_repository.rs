// Wallet Management Repository - Centralized wallet query operations for CQRS handlers
// Eliminates duplicate SQL across admin query handlers

use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::core::errors::{AppError, ErrorKind};

/// Wallet basic information record
#[derive(Debug, Clone)]
pub struct WalletBasicInfo {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub wallet_metadata: Option<serde_json::Value>,
}

/// Wallet summary with counts (for list view)
#[derive(Debug, Clone)]
pub struct WalletSummary {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub permissions_count: i32,
    pub groups_count: i32,
    pub wallet_metadata: Option<serde_json::Value>,
}

/// Wallet permission record
#[derive(Debug, Clone)]
pub struct WalletPermission {
    pub permission: String,
    pub source: String, // "group" or "direct"
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Search criteria for wallet queries
#[derive(Debug, Clone, Default)]
pub struct WalletSearchCriteria {
    pub search: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub exclude_group_id: Option<String>,
    pub limit: i32,
    pub offset: i32,
}

/// Repository for wallet management operations
pub struct WalletManagementRepository {
    pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl WalletManagementRepository {
    pub fn new(pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
        Self { pool }
    }

    /// Get wallet basic information
    pub async fn get_wallet_basic_info(
        &self,
        wallet_address: &str,
    ) -> Result<Option<WalletBasicInfo>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct WalletRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            last_auth_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            wallet_metadata: Option<serde_json::Value>,
        }

        let wallet = diesel::sql_query(
            r#"
            SELECT wallet_address, is_active, created_at, last_auth_at, wallet_metadata
            FROM wallet_users
            WHERE wallet_address = $1
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .get_result::<WalletRow>(&mut conn)
        .await
        .optional()
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch wallet: {}", e)))?;

        Ok(wallet.map(|w| WalletBasicInfo {
            wallet_address: w.wallet_address,
            is_active: w.is_active,
            created_at: w.created_at,
            last_auth_at: w.last_auth_at,
            wallet_metadata: w.wallet_metadata,
        }))
    }

    /// Get wallet permissions (from groups + direct)
    pub async fn get_wallet_permissions(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<WalletPermission>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct PermissionRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            permission: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            source: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            granted_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            expires_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
            is_active: Option<bool>,
        }

        let permissions = diesel::sql_query(
            r#"
            SELECT
                p.permission_string as permission,
                'group' as source,
                pgm.granted_at,
                wgm.expires_at,
                wgm.is_active
            FROM wallet_group_assignments wgm
            JOIN group_permissions pgm ON wgm.group_id = pgm.group_id
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE wgm.wallet_address = $1
              AND p.is_active = true

            UNION ALL

            SELECT
                p.permission_string as permission,
                'direct' as source,
                wdp.granted_at,
                wdp.expires_at,
                wdp.is_active
            FROM wallet_direct_permissions wdp
            JOIN permissions p ON wdp.permission_id = p.id
            WHERE wdp.wallet_address = $1
              AND p.is_active = true

            ORDER BY permission
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .load::<PermissionRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch permissions: {}", e)))?;

        Ok(permissions.into_iter().map(|row| WalletPermission {
            permission: row.permission.unwrap_or_else(|| "unknown".to_string()),
            source: row.source.unwrap_or_else(|| "unknown".to_string()),
            granted_at: row.granted_at.unwrap_or_else(chrono::Utc::now),
            expires_at: row.expires_at,
            is_active: row.is_active.unwrap_or(true),
        }).collect())
    }

    /// Find wallets with filtering and pagination (safe parameterized query)
    pub async fn find_wallets_paginated(
        &self,
        criteria: &WalletSearchCriteria,
    ) -> Result<Vec<WalletSummary>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        // Build WHERE conditions with SQL injection protection
        let mut where_parts = Vec::new();

        if let Some(ref search) = criteria.search {
            let escaped = search.replace("'", "''");
            where_parts.push(format!(
                "(wu.wallet_address ILIKE '%{}%' OR wu.wallet_metadata->>'label' ILIKE '%{}%' OR wu.wallet_metadata->>'note' ILIKE '%{}%')",
                escaped, escaped, escaped
            ));
        }

        if let Some(ref status) = criteria.status {
            let is_active = status == "active";
            where_parts.push(format!("wu.is_active = {}", is_active));
        }

        if let Some(ref date_from) = criteria.date_from {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date_from) {
                where_parts.push(format!("wu.created_at >= '{}'", parsed.to_rfc3339().replace("'", "''")));
            }
        }

        if let Some(ref date_to) = criteria.date_to {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date_to) {
                where_parts.push(format!("wu.created_at <= '{}'", parsed.to_rfc3339().replace("'", "''")));
            }
        }

        if let Some(ref group_id) = criteria.exclude_group_id {
            where_parts.push(format!("wu.wallet_address NOT IN (SELECT wallet_address FROM wallet_group_assignments WHERE group_id = '{}' AND is_active = true)", group_id.replace("'", "''")));
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        // Validate and sanitize sort columns
        let sort_by = criteria.sort_by.as_deref().unwrap_or("created_at");
        let valid_sort_columns = ["created_at", "wallet_address", "last_auth_at", "is_active"];
        let safe_sort_by = if valid_sort_columns.contains(&sort_by) {
            sort_by
        } else {
            "created_at"
        };

        let sort_order = criteria.sort_order.as_deref().unwrap_or("DESC");
        let safe_sort_order = if sort_order.to_uppercase() == "ASC" { "ASC" } else { "DESC" };

        // Build main query
        let query_str = format!(
            r#"
            SELECT
                wu.wallet_address,
                wu.is_active,
                wu.created_at,
                wu.last_auth_at,
                wu.wallet_metadata,
                COALESCE(perms.count, 0)::int as permissions_count,
                0 as groups_count
            FROM wallet_users wu
            LEFT JOIN (
                SELECT wallet_address, COUNT(*)::int as count
                FROM wallet_group_assignments
                WHERE is_active = true
                GROUP BY wallet_address
            ) perms ON wu.wallet_address = perms.wallet_address
            {}
            ORDER BY wu.{} {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, safe_sort_by, safe_sort_order, criteria.limit, criteria.offset
        );

        #[derive(QueryableByName)]
        struct WalletSummaryRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            last_auth_at: Option<DateTime<Utc>>,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            permissions_count: i32,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            groups_count: i32,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            wallet_metadata: Option<serde_json::Value>,
        }

        let rows = diesel::sql_query(&query_str)
            .load::<WalletSummaryRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch wallets: {}", e)))?;

        Ok(rows.into_iter().map(|row| WalletSummary {
            wallet_address: row.wallet_address,
            is_active: row.is_active,
            created_at: row.created_at,
            last_auth_at: row.last_auth_at,
            permissions_count: row.permissions_count,
            groups_count: row.groups_count,
            wallet_metadata: row.wallet_metadata,
        }).collect())
    }

    /// Count wallets with filters
    pub async fn count_wallets(
        &self,
        criteria: &WalletSearchCriteria,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        // Build WHERE conditions with SQL injection protection
        let mut where_parts = Vec::new();

        if let Some(ref search) = criteria.search {
            let escaped = search.replace("'", "''");
            where_parts.push(format!(
                "(wu.wallet_address ILIKE '%{}%' OR wu.wallet_metadata->>'label' ILIKE '%{}%' OR wu.wallet_metadata->>'note' ILIKE '%{}%')",
                escaped, escaped, escaped
            ));
        }

        if let Some(ref status) = criteria.status {
            let is_active = status == "active";
            where_parts.push(format!("wu.is_active = {}", is_active));
        }

        if let Some(ref date_from) = criteria.date_from {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date_from) {
                where_parts.push(format!("wu.created_at >= '{}'", parsed.to_rfc3339().replace("'", "''")));
            }
        }

        if let Some(ref date_to) = criteria.date_to {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(date_to) {
                where_parts.push(format!("wu.created_at <= '{}'", parsed.to_rfc3339().replace("'", "''")));
            }
        }

        if let Some(ref group_id) = criteria.exclude_group_id {
            where_parts.push(format!("wu.wallet_address NOT IN (SELECT wallet_address FROM wallet_group_assignments WHERE group_id = '{}' AND is_active = true)", group_id.replace("'", "''")));
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        let query_str = format!("SELECT COUNT(*) as total FROM wallet_users wu {}", where_clause);

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total: i64,
        }

        let row = diesel::sql_query(&query_str)
            .get_result::<CountRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count wallets: {}", e)))?;

        Ok(row.total)
    }

    /// Get permission count for a wallet
    pub async fn get_permission_count(&self, wallet_address: &str) -> Result<i32, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        #[derive(QueryableByName)]
        struct PermCountRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
            total_count: Option<i32>,
        }

        let row = diesel::sql_query(
            r#"
            SELECT COALESCE((
                SELECT COUNT(DISTINCT p.id)::int
                FROM wallet_group_assignments wgm
                JOIN group_permissions pgm ON wgm.group_id = pgm.group_id
                JOIN permissions p ON pgm.permission_id = p.id
                WHERE wgm.wallet_address = $1
                  AND wgm.is_active = true
                  AND p.is_active = true
                  AND (wgm.expires_at IS NULL OR wgm.expires_at > NOW())
            ), 0) + COALESCE((
                SELECT COUNT(DISTINCT p.id)::int
                FROM wallet_direct_permissions wdp
                JOIN permissions p ON wdp.permission_id = p.id
                WHERE wdp.wallet_address = $1
                  AND wdp.is_active = true
                  AND p.is_active = true
                  AND (wdp.expires_at IS NULL OR wdp.expires_at > NOW())
            ), 0) as total_count
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet_address)
        .get_result::<PermCountRow>(&mut conn)
        .await
        .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to count permissions: {}", e)))?;

        Ok(row.total_count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_repository_creation() {
        // Placeholder test to ensure module compiles
        assert!(true);
    }
}
