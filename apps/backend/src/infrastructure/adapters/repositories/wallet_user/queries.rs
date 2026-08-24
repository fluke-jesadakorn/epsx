// WalletUserSearchPort implementation — find_by_* search methods
//
// BIG-BANG: migrated to sqlx (real).

use super::{WalletUserQueryResult, WalletUserRepositoryAdapter};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;
use crate::domain::wallet_management::{
    aggregates::{WalletMetadata, WalletUser},
    repository_ports::{WalletUserSearchCriteria, WalletUserSearchPort, WalletUserSearchResult},
    value_objects::{Permission, PermissionType, WalletAddress},
};
use crate::prelude::*;
use std::collections::HashSet;
use tracing::error;

fn build_user(row: WalletUserQueryResult) -> Option<WalletUser> {
    let wallet_addr = WalletAddress::new(row.wallet_address).ok()?;
    let metadata = WalletMetadata::from_json(row.wallet_metadata).unwrap_or_default();
    Some(WalletUser::load(WalletUserLoadParams {
        wallet_address: wallet_addr,
        is_active: row.is_active,
        permissions: HashSet::new(),
        plans: HashSet::new(),
        wallet_metadata: metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
        last_auth_at: row.last_auth_at,
        version: 1,
    }))
}

#[async_trait]
impl WalletUserSearchPort for WalletUserRepositoryAdapter {
    async fn find_by_permission(&self, permission: &Permission) -> AppResult<Vec<WalletUser>> {
        let permission_str = permission.as_str();

        let rows: Vec<WalletUserQueryResult> = sqlx::query_as(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true AND wallet_address IN (
                SELECT wga.wallet_address
                FROM wallet_plan_assignments wga
                JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
                JOIN permissions p1 ON pgm.permission_id = p1.id
                WHERE p1.permission_string = $1 AND p1.is_active = true AND wga.is_active = true
                UNION
                SELECT wdp.wallet_address
                FROM wallet_direct_permissions wdp
                JOIN permissions p2 ON wdp.permission_id = p2.id
                WHERE p2.permission_string = $1 AND p2.is_active = true AND wdp.is_active = true
            )
            "#,
        )
        .bind(permission_str)
        .fetch_all(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to find users by permission {}: {}", permission_str, e);
            AppError::database_error(e.to_string()).with_component("wallet_user_repository")
        })?;

        Ok(rows.into_iter().filter_map(build_user).collect())
    }

    async fn find_by_permission_type(
        &self,
        permission_type: &PermissionType,
    ) -> AppResult<Vec<WalletUser>> {
        let type_filter = match permission_type {
            PermissionType::Manual => "manual",
            PermissionType::NftGated { .. } => "nft_gated",
            PermissionType::TokenGated { .. } => "token_gated",
            PermissionType::DaoGovernance { .. } => "dao_governance",
        };

        let rows: Vec<WalletUserQueryResult> = sqlx::query_as(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true AND wallet_address IN (
                SELECT wga.wallet_address
                FROM wallet_plan_assignments wga
                JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id
                JOIN permissions p ON pgm.permission_id = p.id
                WHERE p.permission_type = $1 AND p.is_active = true AND wga.is_active = true
                UNION
                SELECT wdp.wallet_address
                FROM wallet_direct_permissions wdp
                JOIN permissions p ON wdp.permission_id = p.id
                WHERE p.permission_type = $1 AND p.is_active = true AND wdp.is_active = true
            )
            "#,
        )
        .bind(type_filter)
        .fetch_all(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to find users by permission type {}: {}", type_filter, e);
            AppError::database_error(e.to_string())
        })?;

        Ok(rows.into_iter().filter_map(build_user).collect())
    }

    async fn search(&self, criteria: WalletUserSearchCriteria) -> AppResult<WalletUserSearchResult> {
        let mut sql = String::from(
            "SELECT wallet_address, is_active, wallet_metadata, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users WHERE TRUE",
        );

        if criteria.active_only.unwrap_or(true) {
            sql.push_str(" AND is_active = TRUE");
        }

        if let Some(addr) = &criteria.wallet_address {
            sql.push_str(" AND LOWER(wallet_address) = LOWER($1)");
            return self.fetch_search_results_with_wallet(sql, addr).await;
        }

        if let Some(p) = &criteria.permission {
            sql.push_str(&format!(
                " AND wallet_address IN (SELECT wga.wallet_address FROM wallet_plan_assignments wga \
                 JOIN plan_permissions pgm ON wga.plan_id = pgm.plan_id JOIN permissions p ON pgm.permission_id = p.id \
                 WHERE p.permission_string = '{}' AND p.is_active = TRUE AND wga.is_active = TRUE \
                 UNION SELECT wdp.wallet_address FROM wallet_direct_permissions wdp \
                 JOIN permissions p ON wdp.permission_id = p.id \
                 WHERE p.permission_string = '{}' AND p.is_active = TRUE AND wdp.is_active = TRUE)",
                p, p
            ));
        }

        let limit = criteria.limit.unwrap_or(50).min(1000);
        let offset = criteria.offset.unwrap_or(0);
        sql.push_str(&format!(
            " ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        ));

        let rows: Vec<WalletUserQueryResult> = sqlx::query_as(&sql)
            .fetch_all(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(WalletUserSearchResult {
            total: rows.len() as i64,
            items: rows.into_iter().filter_map(build_user).collect(),
        })
    }

    async fn count(&self, criteria: WalletUserSearchCriteria) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_users WHERE is_active = TRUE")
            .fetch_one(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.0)
    }
}

impl WalletUserRepositoryAdapter {
    async fn fetch_search_results_with_wallet(
        &self,
        sql: String,
        addr: &str,
    ) -> AppResult<WalletUserSearchResult> {
        let rows: Vec<WalletUserQueryResult> = sqlx::query_as(&sql)
            .bind(addr)
            .fetch_all(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(WalletUserSearchResult {
            total: rows.len() as i64,
            items: rows.into_iter().filter_map(build_user).collect(),
        })
    }
}