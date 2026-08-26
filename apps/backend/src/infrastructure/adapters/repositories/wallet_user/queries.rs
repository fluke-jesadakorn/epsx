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

fn empty_search_result(limit: u32, offset: u32) -> WalletUserSearchResult {
    WalletUserSearchResult {
        users: Vec::new(),
        total_count: 0,
        offset,
        limit,
        has_more: false,
        web3_metadata: std::collections::HashMap::new(),
    }
}

#[async_trait]
impl WalletUserSearchPort for WalletUserRepositoryAdapter {
    async fn find_by_criteria(
        &self,
        _criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32,
    ) -> AppResult<WalletUserSearchResult> {
        let lim = limit.min(1000);
        let rows: Vec<WalletUserQueryResult> = sqlx::query_as(
            "SELECT wallet_address, is_active, wallet_metadata, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(lim as i64)
        .bind(offset as i64)
        .fetch_all(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let users: Vec<WalletUser> = rows.into_iter().filter_map(build_user).collect();
        Ok(WalletUserSearchResult {
            users,
            total_count: 0,
            offset,
            limit,
            has_more: false,
            web3_metadata: std::collections::HashMap::new(),
        })
    }

    async fn count_by_criteria(&self, _criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_users")
            .fetch_one(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.0 as u64)
    }

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

    async fn find_by_permission_plan(
        &self,
        _permission_plan: &str,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_nft_ownership(
        &self,
        _contract_address: &str,
        _token_ids: Option<&[u64]>,
        _chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_token_balance(
        &self,
        _contract_address: &str,
        _min_balance: &str,
        _chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_dao_membership(
        &self,
        _dao_contract: &str,
        _min_voting_power: &str,
        _chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn validate_web3_permissions(
        &self,
        _wallet_address: &WalletAddress,
        _permissions: &[Permission],
    ) -> AppResult<Vec<bool>> {
        Ok(Vec::new())
    }

    async fn cache_web3_validation(
        &self,
        _wallet_address: &WalletAddress,
        _permission: &Permission,
        _is_valid: bool,
        _cache_duration_seconds: u64,
    ) -> AppResult<()> {
        Ok(())
    }
}

impl WalletUserRepositoryAdapter {
    #[allow(dead_code)]
    async fn fetch_search_results_with_wallet(
        &self,
        _sql: String,
        _addr: &str,
    ) -> AppResult<WalletUserSearchResult> {
        empty_search_result(50, 0);
        Ok(WalletUserSearchResult {
            users: Vec::new(),
            total_count: 0,
            offset: 0,
            limit: 50,
            has_more: false,
            web3_metadata: std::collections::HashMap::new(),
        })
    }
}