// Wallet Management Repository - Centralized wallet query operations for CQRS handlers
// Eliminates duplicate SQL across admin query handlers

use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use crate::infrastructure::database::diesel_connection_manager::TlsPool;

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
    pub plans_count: i32,
    pub plan_name: Option<String>,
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
    pub exclude_plan_id: Option<String>,
    pub limit: i32,
    pub offset: i32,
}

/// Repository for wallet management operations
pub struct WalletManagementRepository {
    pool: Arc<&'static TlsPool>,
}

impl WalletManagementRepository {
    pub fn new(pool: Arc<&'static TlsPool>) -> Self {
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

    /// Get wallet permissions (from plans + direct)
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
                'plan' as source,
                pgm.granted_at,
                wgm.expires_at,
                wgm.is_active
            FROM wallet_plan_assignments wgm
            JOIN plan_permissions pgm ON wgm.plan_id = pgm.plan_id
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

    /// Find wallets with filtering and pagination (parameterized query)
    pub async fn find_wallets_paginated(
        &self,
        criteria: &WalletSearchCriteria,
    ) -> Result<Vec<WalletSummary>, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        // Validate and sanitize sort columns (whitelist — safe for format!())
        let sort_by = criteria.sort_by.as_deref().unwrap_or("created_at");
        let valid_sort_columns = ["created_at", "wallet_address", "last_auth_at", "is_active"];
        let safe_sort_by = if valid_sort_columns.contains(&sort_by) {
            sort_by
        } else {
            "created_at"
        };

        let sort_order = criteria.sort_order.as_deref().unwrap_or("DESC");
        let safe_sort_order = if sort_order.to_uppercase() == "ASC" { "ASC" } else { "DESC" };

        // Prepare bind params — user input is NEVER interpolated
        let search_pattern = criteria.search.as_ref().map(|s| format!("%{}%", s));
        let is_active_filter: Option<bool> = criteria.status.as_ref().and_then(|s| match s.as_str() {
            "active" => Some(true),
            "disabled" => Some(false),
            _ => None,
        });
        let date_from: Option<DateTime<Utc>> = criteria.date_from.as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc));
        let date_to: Option<DateTime<Utc>> = criteria.date_to.as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc));
        let exclude_plan = criteria.exclude_plan_id.clone();

        // Only ORDER BY uses format!() — values are whitelisted above
        // Use LEFT JOIN + GROUP BY to avoid 3 correlated subqueries per row
        let query_str = format!(
            r#"
            SELECT
                wu.wallet_address,
                wu.is_active,
                wu.created_at,
                wu.last_auth_at,
                wu.wallet_metadata,
                COALESCE(agg.plans_count, 0) as plans_count,
                COALESCE(agg.plans_count, 0) as permissions_count,
                top_plan.plan_name as plan_name
            FROM wallet_users wu
            LEFT JOIN (
                SELECT wallet_address, COUNT(*)::int as plans_count
                FROM wallet_plan_assignments
                WHERE is_active = true
                GROUP BY wallet_address
            ) agg ON agg.wallet_address = wu.wallet_address
            LEFT JOIN (
                SELECT DISTINCT ON (wpa.wallet_address)
                    wpa.wallet_address,
                    pp.name as plan_name
                FROM wallet_plan_assignments wpa
                JOIN plans pp ON pp.id = wpa.plan_id
                WHERE wpa.is_active = true
                ORDER BY wpa.wallet_address, pp.tier_level DESC NULLS LAST
            ) top_plan ON top_plan.wallet_address = wu.wallet_address
            WHERE ($1::text IS NULL OR wu.wallet_address ILIKE $1 OR wu.wallet_metadata->>'label' ILIKE $1 OR wu.wallet_metadata->>'note' ILIKE $1)
              AND ($2::bool IS NULL OR wu.is_active = $2)
              AND ($3::timestamptz IS NULL OR wu.created_at >= $3)
              AND ($4::timestamptz IS NULL OR wu.created_at <= $4)
              AND ($5::text IS NULL OR NOT EXISTS (SELECT 1 FROM wallet_plan_assignments WHERE wallet_address = wu.wallet_address AND plan_id = $5::uuid AND is_active = true))
            ORDER BY wu.{} {}
            LIMIT $6 OFFSET $7
            "#,
            safe_sort_by, safe_sort_order
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
            plans_count: i32,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            plan_name: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            wallet_metadata: Option<serde_json::Value>,
        }

        let rows = diesel::sql_query(&query_str)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&search_pattern)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Bool>, _>(&is_active_filter)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&date_from)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&date_to)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&exclude_plan)
            .bind::<diesel::sql_types::BigInt, _>(criteria.limit as i64)
            .bind::<diesel::sql_types::BigInt, _>(criteria.offset as i64)
            .load::<WalletSummaryRow>(&mut conn)
            .await
            .map_err(|e| AppError::new(ErrorKind::DatabaseError, format!("Failed to fetch wallets: {}", e)))?;

        Ok(rows.into_iter().map(|row| WalletSummary {
            wallet_address: row.wallet_address,
            is_active: row.is_active,
            created_at: row.created_at,
            last_auth_at: row.last_auth_at,
            permissions_count: row.permissions_count,
            plans_count: row.plans_count,
            plan_name: row.plan_name,
            wallet_metadata: row.wallet_metadata,
        }).collect())
    }

    /// Count wallets with filters (parameterized query)
    pub async fn count_wallets(
        &self,
        criteria: &WalletSearchCriteria,
    ) -> Result<i64, AppError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::new(ErrorKind::DatabaseError, format!("Failed to get connection: {}", e))
        })?;

        let search_pattern = criteria.search.as_ref().map(|s| format!("%{}%", s));
        let is_active_filter: Option<bool> = criteria.status.as_ref().and_then(|s| match s.as_str() {
            "active" => Some(true),
            "disabled" => Some(false),
            _ => None,
        });
        let date_from: Option<DateTime<Utc>> = criteria.date_from.as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc));
        let date_to: Option<DateTime<Utc>> = criteria.date_to.as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc));
        let exclude_plan = criteria.exclude_plan_id.clone();

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total: i64,
        }

        let row = diesel::sql_query(
            r#"SELECT COUNT(*) as total FROM wallet_users wu
               WHERE ($1::text IS NULL OR wu.wallet_address ILIKE $1 OR wu.wallet_metadata->>'label' ILIKE $1 OR wu.wallet_metadata->>'note' ILIKE $1)
                 AND ($2::bool IS NULL OR wu.is_active = $2)
                 AND ($3::timestamptz IS NULL OR wu.created_at >= $3)
                 AND ($4::timestamptz IS NULL OR wu.created_at <= $4)
                 AND ($5::text IS NULL OR NOT EXISTS (SELECT 1 FROM wallet_plan_assignments WHERE wallet_address = wu.wallet_address AND plan_id = $5::uuid AND is_active = true))"#
        )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&search_pattern)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Bool>, _>(&is_active_filter)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&date_from)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(&date_to)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&exclude_plan)
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
                FROM wallet_plan_assignments wgm
                JOIN plan_permissions pgm ON wgm.plan_id = pgm.plan_id
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
    }
}
