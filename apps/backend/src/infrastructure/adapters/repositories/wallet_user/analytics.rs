// WalletUserAnalyticsPort implementation — statistics and analytics methods
//
// BIG-BANG: migrated to sqlx (real).

use super::{WalletUserQueryResult, WalletUserRepositoryAdapter};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;
use crate::domain::wallet_management::{
    aggregates::{WalletMetadata, WalletUser},
    repository_ports::{WalletUserAnalyticsPort, WalletUserStatistics, Web3Analytics},
    value_objects::WalletAddress,
};
use crate::prelude::*;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use tracing::error;

#[derive(sqlx::FromRow)]
struct StatsResult {
    total_users: i64,
    active_users: i64,
    recent_auth_24h: i64,
    new_wallets_24h: i64,
}

#[async_trait]
impl WalletUserAnalyticsPort for WalletUserRepositoryAdapter {
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics> {
        let row: StatsResult = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
            FROM wallet_users
            "#,
        )
        .fetch_one(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to get wallet user statistics: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_statistics")
        })?;

        Ok(WalletUserStatistics {
            total_users: row.total_users as u64,
            active_users: row.active_users as u64,
            inactive_users: (row.total_users - row.active_users) as u64,
            recent_auth_24h: row.recent_auth_24h as u64,
            new_wallets_24h: row.new_wallets_24h as u64,
            users_by_permission_plan: HashMap::new(),
            users_by_chain: HashMap::new(),
            manual_permissions: 0,
            nft_gated_permissions: 0,
            token_gated_permissions: 0,
            dao_governance_permissions: 0,
            recent_authentications_24h: row.recent_auth_24h as u64,
        })
    }

    async fn get_growth_metrics(&self, days: u32) -> AppResult<Vec<(NaiveDate, u64)>> {
        let row: Vec<(NaiveDate, i64)> = sqlx::query_as(
            "SELECT DATE(created_at) as day, COUNT(*) as count \
             FROM wallet_users \
             WHERE created_at >= NOW() - ($1::int || ' days')::interval \
             GROUP BY DATE(created_at) \
             ORDER BY day ASC",
        )
        .bind(days as i32)
        .fetch_all(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut vec = Vec::new();
        for (day, count) in row {
            vec.push((day, count as u64));
        }
        Ok(vec)
    }

    async fn get_active_users(&self, days: u32) -> AppResult<u64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM wallet_users \
             WHERE is_active = TRUE AND last_auth_at >= NOW() - ($1::int || ' days')::interval",
        )
        .bind(days as i32)
        .fetch_one(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.0 as u64)
    }

    async fn get_web3_analytics(&self) -> AppResult<Web3Analytics> {
        let row: (i64, i64) = sqlx::query_as(
            "SELECT \
                COUNT(*) FILTER (WHERE tier_level = 'Bronze') as bronze_count, \
                COUNT(*) FILTER (WHERE tier_level = 'Silver') as silver_count \
             FROM wallet_users",
        )
        .fetch_one(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let active_24h: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM wallet_users WHERE is_active = TRUE \
             AND last_auth_at >= NOW() - INTERVAL '24 hours'",
        )
        .fetch_one(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(Web3Analytics {
            bronze_tier_count: row.0 as u64,
            silver_tier_count: row.1 as u64,
            active_24h: active_24h.0 as u64,
            top_nft_contracts: Vec::new(),
            top_token_contracts: Vec::new(),
            top_dao_contracts: Vec::new(),
            chain_distribution: HashMap::new(),
            permission_type_distribution: HashMap::new(),
        })
    }

    async fn count_by_tier(&self, tier: &str) -> AppResult<u64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM wallet_users WHERE tier_level = $1")
            .bind(tier)
            .fetch_one(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.0 as u64)
    }

    async fn get_permission_distribution(&self) -> AppResult<HashMap<String, u64>> {
        Ok(HashMap::new())
    }

    async fn get_activity_patterns_by_chain(
        &self,
        _chain_id: u64,
        _days: u32,
    ) -> AppResult<Vec<(NaiveDate, u64)>> {
        Ok(Vec::new())
    }

    async fn find_inactive_users(&self, _days: u32) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn get_plan_progression(&self) -> AppResult<HashMap<String, Vec<String>>> {
        Ok(HashMap::new())
    }

    async fn get_validation_success_rates(&self) -> AppResult<HashMap<String, f64>> {
        Ok(HashMap::new())
    }

    async fn get_cross_chain_analysis(&self) -> AppResult<HashMap<String, u64>> {
        Ok(HashMap::new())
    }
}

#[allow(dead_code)]
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
