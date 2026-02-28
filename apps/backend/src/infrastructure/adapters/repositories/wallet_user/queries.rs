// WalletUserSearchPort implementation — find_by_* search methods

use crate::prelude::*;
use std::collections::HashSet;
use tracing::error;
use diesel_async::RunQueryDsl;
use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::{WalletAddress, Permission, PermissionType},
    repository_ports::{
        WalletUserSearchPort,
        WalletUserSearchCriteria,
        WalletUserSearchResult,
    },
};
use super::{WalletUserRepositoryAdapter, WalletUserQueryResult};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;

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
        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(
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
            "#
        )
        .bind::<diesel::sql_types::Text, _>(permission_str)
        .load::<WalletUserQueryResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to find users by permission {}: {}", permission_str, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        Ok(rows.into_iter().filter_map(build_user).collect())
    }

    async fn find_by_permission_type(&self, permission_type: &PermissionType) -> AppResult<Vec<WalletUser>> {
        let type_filter = match permission_type {
            PermissionType::Manual => "manual",
            PermissionType::NftGated { .. } => "nft_gated",
            PermissionType::TokenGated { .. } => "token_gated",
            PermissionType::DaoGovernance { .. } => "dao_governance",
        };

        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true AND wallet_address IN (
                SELECT wga.wallet_address
                FROM wallet_plan_assignments wga
                JOIN plans pg ON wga.plan_id = pg.id
                WHERE wga.is_active = true AND pg.is_active = true
                  AND (pg.name = $1 OR pg.slug = $1)
            )
            ORDER BY created_at DESC
            "#
        )
        .bind::<diesel::sql_types::Text, _>(type_filter)
        .load::<WalletUserQueryResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to find users by permission type {}: {}", type_filter, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        Ok(rows.into_iter().filter_map(build_user).collect())
    }

    async fn find_by_permission_plan(&self, permission_plan: &str) -> AppResult<Vec<WalletUser>> {
        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true AND wallet_address IN (
                SELECT wga.wallet_address
                FROM wallet_plan_assignments wga
                JOIN plans pg ON wga.plan_id = pg.id
                WHERE wga.is_active = true AND pg.is_active = true
                  AND (pg.name = $1 OR pg.slug = $1)
            )
            ORDER BY created_at DESC
            "#
        )
        .bind::<diesel::sql_types::Text, _>(permission_plan)
        .load::<WalletUserQueryResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to find users by permission plan {}: {}", permission_plan, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_permission_plan")
        })?;

        Ok(rows.into_iter().filter_map(build_user).collect())
    }

    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32,
    ) -> AppResult<WalletUserSearchResult> {
        let mut conn = self.db_pool.conn().await?;
        let wallet_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));
        let is_active = criteria.is_active;
        let permission_plan = criteria.permission_plan.clone();
        let created_after = criteria.created_after;
        let created_before = criteria.created_before;

        let rows = diesel::sql_query(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE ($1::text IS NULL OR wallet_address ILIKE $1)
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::text IS NULL OR plan_metadata ? $3)
              AND ($4::timestamptz IS NULL OR created_at > $4)
              AND ($5::timestamptz IS NULL OR created_at < $5)
            ORDER BY created_at DESC
            LIMIT $6
            OFFSET $7
            "#
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(wallet_pattern)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Bool>, _>(is_active)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(permission_plan)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(created_after)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(created_before)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .bind::<diesel::sql_types::Integer, _>(offset as i32)
        .load::<WalletUserQueryResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to search wallet users: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let users: Vec<WalletUser> = rows.into_iter().filter_map(build_user).collect();
        let total_count = WalletUserSearchPort::count_by_criteria(self, criteria).await?;

        Ok(WalletUserSearchResult::new(users, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
        let mut conn = self.db_pool.conn().await?;
        let wallet_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));
        let is_active = criteria.is_active;

        #[derive(diesel::QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let result = diesel::sql_query(
            r#"SELECT COUNT(*) as count FROM wallet_users
               WHERE ($1::text IS NULL OR wallet_address ILIKE $1)
                 AND ($2::bool IS NULL OR is_active = $2)"#
        )
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(wallet_pattern)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Bool>, _>(is_active)
        .load::<CountResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to count wallet users: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        Ok(result.into_iter().next().map(|r| r.count as u64).unwrap_or(0))
    }

    async fn find_by_nft_ownership(
        &self,
        contract_address: &str,
        _token_ids: Option<&[u64]>,
        chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        tracing::warn!(
            "find_by_nft_ownership not yet implemented for contract {} on chain {}",
            contract_address, chain_id
        );
        Ok(Vec::new())
    }

    async fn find_by_token_balance(
        &self,
        contract_address: &str,
        _min_balance: &str,
        chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        tracing::warn!(
            "find_by_token_balance not yet implemented for contract {} on chain {}",
            contract_address, chain_id
        );
        Ok(Vec::new())
    }

    async fn find_by_dao_membership(
        &self,
        dao_contract: &str,
        _min_voting_power: &str,
        chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        tracing::warn!(
            "find_by_dao_membership not yet implemented for DAO {} on chain {}",
            dao_contract, chain_id
        );
        Ok(Vec::new())
    }

    async fn validate_web3_permissions(
        &self,
        wallet_address: &WalletAddress,
        permissions: &[Permission],
    ) -> AppResult<Vec<bool>> {
        let results = permissions.iter().map(|p| p.is_manual()).collect();
        tracing::info!(
            "Validated {} permissions for wallet {}",
            permissions.len(), wallet_address.as_str()
        );
        Ok(results)
    }

    async fn cache_web3_validation(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        is_valid: bool,
        cache_duration_seconds: u64,
    ) -> AppResult<()> {
        tracing::info!(
            "Would cache validation result for wallet {} permission {}: {} (TTL: {}s)",
            wallet_address.as_str(), permission.as_str(), is_valid, cache_duration_seconds
        );
        Ok(())
    }
}
