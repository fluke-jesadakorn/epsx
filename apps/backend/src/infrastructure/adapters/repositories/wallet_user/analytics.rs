// WalletUserAnalyticsPort implementation — statistics and analytics methods

use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use chrono::{NaiveDate, Utc};
use tracing::{error, info};
use diesel_async::RunQueryDsl;
use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::WalletAddress,
    repository_ports::{
        WalletUserAnalyticsPort,
        WalletUserStatistics,
        Web3Analytics,
    },
};
use super::{WalletUserRepositoryAdapter, WalletUserQueryResult};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;

#[async_trait]
impl WalletUserAnalyticsPort for WalletUserRepositoryAdapter {
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics> {
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct StatsResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_users: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            active_users: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            recent_auth_24h: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            new_wallets_24h: i64,
        }

        let results = diesel::sql_query(
            r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
            FROM wallet_users
            "#
        )
        .load::<StatsResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get wallet user statistics: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_statistics")
        })?;

        let stats = results.into_iter().next().ok_or_else(|| {
            AppError::database_error("No statistics returned".to_string())
                .with_component("wallet_user_repository")
        })?;

        Ok(WalletUserStatistics {
            total_users: stats.total_users as u64,
            active_users: stats.active_users as u64,
            users_by_permission_plan: HashMap::new(),
            users_by_chain: HashMap::new(),
            manual_permissions: 0,
            nft_gated_permissions: 0,
            token_gated_permissions: 0,
            dao_governance_permissions: 0,
            recent_authentications_24h: stats.recent_auth_24h as u64,
            new_wallets_24h: stats.new_wallets_24h as u64,
        })
    }

    async fn get_web3_analytics(&self) -> AppResult<Web3Analytics> {
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct PermissionTypeRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_type: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let permission_type_rows = diesel::sql_query(
            r#"
            SELECT
                permission_type,
                COUNT(DISTINCT p.id) as count
            FROM permissions p
            WHERE p.is_active = true
            GROUP BY permission_type
            "#
        )
        .load::<PermissionTypeRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get permission type distribution: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_web3_analytics - permission_type_distribution")
        })?;

        let mut permission_type_distribution = HashMap::new();
        for row in permission_type_rows {
            permission_type_distribution.insert(row.permission_type, row.count as u64);
        }

        #[derive(diesel::QueryableByName)]
        struct ChainRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            chain_id: Option<String>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let chain_rows = diesel::sql_query(
            r#"
            SELECT chain_id, count
            FROM mv_web3_chain_distribution
            ORDER BY count DESC
            "#
        )
        .load::<ChainRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get chain distribution: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_web3_analytics - chain_distribution")
        })?;

        let mut chain_distribution = HashMap::new();
        for row in chain_rows {
            if let Some(chain_id_str) = row.chain_id {
                if let Ok(chain_id) = chain_id_str.parse::<u64>() {
                    chain_distribution.insert(chain_id, row.count as u64);
                }
            }
        }

        info!(
            "Generated Web3 analytics: {} permission types, {} chains",
            permission_type_distribution.len(), chain_distribution.len()
        );

        #[derive(diesel::QueryableByName)]
        struct ContractRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            contract_address: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            user_count: i64,
        }

        let nft_rows: Vec<ContractRow> = diesel::sql_query(
            r#"
            SELECT
                jsonb_array_elements_text(wallet_metadata->'verified_networks') as contract_address,
                COUNT(DISTINCT wallet_address) as user_count
            FROM wallet_users
            WHERE wallet_metadata ? 'nft_permissions_cache'
              AND wallet_metadata->'nft_permissions_cache' IS NOT NULL
              AND is_active = true
            GROUP BY contract_address
            ORDER BY user_count DESC
            LIMIT 10
            "#
        )
        .load::<ContractRow>(&mut conn)
        .await
        .unwrap_or_default();

        let token_rows: Vec<ContractRow> = diesel::sql_query(
            r#"
            SELECT
                jsonb_array_elements_text(wallet_metadata->'verified_networks') as contract_address,
                COUNT(DISTINCT wallet_address) as user_count
            FROM wallet_users
            WHERE wallet_metadata ? 'token_permissions_cache'
              AND wallet_metadata->'token_permissions_cache' IS NOT NULL
              AND is_active = true
            GROUP BY contract_address
            ORDER BY user_count DESC
            LIMIT 10
            "#
        )
        .load::<ContractRow>(&mut conn)
        .await
        .unwrap_or_default();

        let dao_rows: Vec<ContractRow> = diesel::sql_query(
            r#"
            SELECT
                jsonb_array_elements_text(wallet_metadata->'dao_memberships') as contract_address,
                COUNT(DISTINCT wallet_address) as user_count
            FROM wallet_users
            WHERE wallet_metadata ? 'dao_memberships'
              AND jsonb_array_length(wallet_metadata->'dao_memberships') > 0
              AND is_active = true
            GROUP BY contract_address
            ORDER BY user_count DESC
            LIMIT 10
            "#
        )
        .load::<ContractRow>(&mut conn)
        .await
        .unwrap_or_default();

        Ok(Web3Analytics {
            top_nft_contracts: nft_rows.into_iter().map(|r| (r.contract_address, r.user_count as u64)).collect(),
            top_token_contracts: token_rows.into_iter().map(|r| (r.contract_address, r.user_count as u64)).collect(),
            top_dao_contracts: dao_rows.into_iter().map(|r| (r.contract_address, r.user_count as u64)).collect(),
            chain_distribution,
            permission_type_distribution,
        })
    }

    async fn get_permission_distribution(&self) -> AppResult<HashMap<String, u64>> {
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct PermissionDistRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_string: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            user_count: i64,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT
                p.permission_string,
                COUNT(DISTINCT COALESCE(wgm.wallet_address, wdp.wallet_address)) as user_count
            FROM permissions p
            LEFT JOIN plan_permissions pgm ON p.id = pgm.permission_id
            LEFT JOIN wallet_plan_assignments wgm ON pgm.plan_id = wgm.plan_id AND wgm.is_active = true
            LEFT JOIN wallet_direct_permissions wdp ON p.id = wdp.permission_id AND wdp.is_active = true
            WHERE p.is_active = true
            GROUP BY p.permission_string
            HAVING COUNT(DISTINCT COALESCE(wgm.wallet_address, wdp.wallet_address)) > 0
            ORDER BY user_count DESC
            "#
        )
        .load::<PermissionDistRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get permission distribution: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_permission_distribution")
        })?;

        let distribution: HashMap<String, u64> = rows.into_iter()
            .map(|r| (r.permission_string, r.user_count as u64))
            .collect();

        info!("Retrieved permission distribution for {} permissions", distribution.len());
        Ok(distribution)
    }

    async fn get_activity_patterns_by_chain(
        &self,
        chain_id: u64,
        days: u32,
    ) -> AppResult<Vec<(NaiveDate, u64)>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct ActivityRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)]
            auth_date: Option<NaiveDate>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            daily_active_users: i64,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT
                DATE(last_auth_at) as auth_date,
                COUNT(DISTINCT wallet_address) as daily_active_users
            FROM wallet_users
            WHERE is_active = true
              AND last_auth_at >= $1
              AND (wallet_metadata->>'primary_chain_id')::bigint = $2
            GROUP BY DATE(last_auth_at)
            ORDER BY auth_date DESC
            "#
        )
        .bind::<diesel::sql_types::Timestamptz, _>(cutoff_date)
        .bind::<diesel::sql_types::BigInt, _>(chain_id as i64)
        .load::<ActivityRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get activity patterns for chain {}: {}", chain_id, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("get_activity_patterns_by_chain(chain={}, days={})", chain_id, days))
        })?;

        let patterns: Vec<(NaiveDate, u64)> = rows.into_iter()
            .filter_map(|row| row.auth_date.map(|d| (d, row.daily_active_users as u64)))
            .collect();

        info!("Retrieved {} activity patterns for chain {} over {} days", patterns.len(), chain_id, days);
        Ok(patterns)
    }

    async fn find_inactive_users(&self, days: u32) -> AppResult<Vec<WalletUser>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (last_auth_at IS NULL OR last_auth_at < $1)
            ORDER BY last_auth_at ASC NULLS FIRST
            "#
        )
        .bind::<diesel::sql_types::Timestamptz, _>(cutoff_date)
        .load::<WalletUserQueryResult>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to find inactive users: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    users.push(WalletUser::load(WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        plans: HashSet::new(),
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    }));
                }
            }
        }

        Ok(users)
    }

    async fn get_plan_progression(&self) -> AppResult<HashMap<String, Vec<String>>> {
        let mut conn = self.db_pool.conn().await?;

        #[derive(diesel::QueryableByName)]
        struct ProgressionRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_plan: String,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT DISTINCT ON (wallet_address, resource_id)
                wallet_address,
                resource_id as permission_plan,
                created_at as event_timestamp
            FROM audit_logs
            WHERE action = 'plan_assigned'
              AND resource_type = 'plan'
              AND resource_id IS NOT NULL
            ORDER BY wallet_address, resource_id, created_at ASC
            "#
        )
        .load::<ProgressionRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get plan progression: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let mut progression: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            if !row.wallet_address.is_empty() && !row.permission_plan.is_empty() {
                progression.entry(row.wallet_address).or_default().push(row.permission_plan);
            }
        }

        info!("Retrieved plan progression for {} wallets", progression.len());
        Ok(progression)
    }

    async fn get_validation_success_rates(&self) -> AppResult<HashMap<String, f64>> {
        Ok(HashMap::new())
    }

    async fn get_cross_chain_analysis(&self) -> AppResult<HashMap<String, u64>> {
        Ok(HashMap::new())
    }
}
