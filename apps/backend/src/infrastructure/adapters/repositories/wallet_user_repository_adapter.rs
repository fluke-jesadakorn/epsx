// Wallet User Repository Adapter (Infrastructure Layer)
// Implements WalletUserRepositoryPort using Diesel and PostgreSQL for Web3-first authentication

use crate::prelude::*;

use std::collections::{HashMap, HashSet};
use chrono::{NaiveDate, Utc};
use tracing::{error, info, warn};

// Diesel imports
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use crate::schemas::primary::wallet_users;
use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, NewWalletUserDb};

// Define PostgreSQL LOWER() for type-safe case-insensitive queries
diesel::sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::{WalletAddress, Permission, PermissionType},
    repository_ports::{
        WalletUserRepositoryPort,
        WalletUserSearchPort,
        WalletUserAnalyticsPort,
        WalletUserSearchCriteria,
        WalletUserSearchResult,
        WalletUserStatistics,
        Web3Analytics,
    },
};

// Query result struct for raw SQL permission queries
#[derive(diesel::QueryableByName)]
struct WalletUserQueryResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    wallet_address: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    wallet_metadata: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// PostgreSQL implementation of WalletUserRepositoryPort using Diesel
#[derive(Clone)]
pub struct WalletUserRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl WalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserRepositoryPort for WalletUserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>> {
        let mut conn = self.db_pool.conn().await?;

        // Diesel query: case-insensitive wallet address lookup using type-safe lower()
        let wallet_addr_lower = wallet_address.as_str().to_lowercase();
        let db_user = wallet_users::table
            .filter(lower(wallet_users::wallet_address).eq(wallet_addr_lower))
            .select(WalletUserDb::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find wallet user by address {}: {}", wallet_address.as_str(), e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(format!("find_by_wallet({})", wallet_address.as_str()))
            })?;

        if let Some(row) = db_user {
            let wallet_addr = WalletAddress::new(row.wallet_address)
                .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e))
                    .with_component("wallet_user_repository"))?;

            let permission_set: HashSet<Permission> = HashSet::new();
            let permission_plan_set: HashSet<String> = HashSet::new();

            let metadata = WalletMetadata::from_json(row.wallet_metadata)
                .map_err(|e| AppError::validation_error(format!("Invalid wallet metadata: {}", e))
                    .with_component("wallet_user_repository"))?;

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address: wallet_addr,
                is_active: row.is_active,
                permissions: permission_set,
                plans: permission_plan_set,
                wallet_metadata: metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at: row.last_auth_at,
                version: 1, // Current wallet user version
            });

            Ok(Some(wallet))
        } else {
            Ok(None)
        }
    }

    async fn find_by_wallets(&self, wallet_addresses: &[WalletAddress]) -> AppResult<Vec<WalletUser>> {
        if wallet_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.db_pool.conn().await?;

        let addresses_lower: Vec<String> = wallet_addresses.iter()
            .map(|w| w.as_str().to_lowercase())
            .collect();

        let db_users = wallet_users::table
            .filter(lower(wallet_users::wallet_address).eq_any(addresses_lower))
            .order(wallet_users::created_at.desc())
            .select(WalletUserDb::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find wallet users by addresses: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(format!("find_by_wallets({} addresses)", wallet_addresses.len()))
            })?;

        let mut users = Vec::new();
        for row in db_users {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                let permission_set: HashSet<Permission> = HashSet::new();
                let permission_plan_set: HashSet<String> = HashSet::new();

                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: permission_set,
                        plans: permission_plan_set,
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    });
                    users.push(wallet);
                }
            }
        }

        Ok(users)
    }

    async fn save(&self, user: &WalletUser) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        let metadata_json = user.wallet_metadata().to_json()
            .map_err(|e| AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository"))?;

        let new_user = NewWalletUserDb {
            wallet_address: user.wallet_address().as_str().to_lowercase(),
            is_active: user.is_active(),
            tier_level: "free".to_string(), // Default tier
            wallet_metadata: metadata_json.clone(),
        };

        diesel::insert_into(wallet_users::table)
            .values(&new_user)
            .on_conflict(wallet_users::wallet_address)
            .do_update()
            .set((
                wallet_users::is_active.eq(user.is_active()),
                wallet_users::wallet_metadata.eq(metadata_json),
                wallet_users::updated_at.eq(user.updated_at()),
                wallet_users::last_auth_at.eq(user.last_auth_at()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save wallet user {}: {}", user.wallet_address().as_str(), e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(format!("save({})", user.wallet_address().as_str()))
            })?;

        info!("Saved wallet user: {}", user.wallet_address().as_str());
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        let wallet_addr_lower = wallet_address.as_str().to_lowercase();
        let rows_affected = diesel::delete(
            wallet_users::table.filter(
                lower(wallet_users::wallet_address).eq(wallet_addr_lower)
            )
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to delete wallet user {}: {}", wallet_address.as_str(), e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        if rows_affected > 0 {
            info!("Deleted wallet user: {}", wallet_address.as_str());
        } else {
            warn!("No wallet user found to delete: {}", wallet_address.as_str());
        }

        Ok(())
    }

    async fn find_eligible_for_web3_permissions(&self, chain_id: u64) -> AppResult<Vec<WalletUser>> {
        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (wallet_metadata->>'primary_chain_id')::bigint = $1
            "#
        )
            .bind::<diesel::sql_types::BigInt, _>(chain_id as i64)
            .load::<WalletUserQueryResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find eligible users for chain {}: {}", chain_id, e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
            })?;

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        plans: HashSet::new(),
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    });
                    users.push(wallet);
                }
            }
        }

        Ok(users)
    }

    async fn save_batch(&self, users: &[WalletUser]) -> AppResult<()> {
        if users.is_empty() {
            return Ok(());
        }

        let mut conn = self.db_pool.conn().await?;

        for user in users {
            let metadata_json = user.wallet_metadata().to_json()
                .map_err(|e| AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository"))?;

            let new_user = NewWalletUserDb {
                wallet_address: user.wallet_address().as_str().to_lowercase(),
                is_active: user.is_active(),
                tier_level: "free".to_string(),
                wallet_metadata: metadata_json.clone(),
            };

            diesel::insert_into(wallet_users::table)
                .values(&new_user)
                .on_conflict(wallet_users::wallet_address)
                .do_update()
                .set((
                    wallet_users::is_active.eq(user.is_active()),
                    wallet_users::wallet_metadata.eq(metadata_json),
                    wallet_users::updated_at.eq(user.updated_at()),
                    wallet_users::last_auth_at.eq(user.last_auth_at()),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    error!("Failed to save wallet user in batch {}: {}", user.wallet_address().as_str(), e);
                    AppError::database_error(e.to_string())
                        .with_component("wallet_user_repository")
                })?;
        }

        info!("Saved batch of {} wallet users", users.len());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        use diesel::dsl::sql;

        let _: i32 = diesel::select(sql::<diesel::sql_types::Integer>("SELECT 1"))
            .get_result(&mut conn)
            .await
            .map_err(|e| {
                error!("Health check failed: {}", e);
                AppError::database_error(format!("Database health check error: {}", e))
                    .with_component("wallet_user_repository")
                    .with_operation("health_check")
            })?;

        Ok(())
    }

    async fn cleanup_expired_permissions(&self) -> AppResult<u32> {
        warn!("cleanup_expired_permissions not yet implemented");
        Ok(0)
    }
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

        let mut users = Vec::new();
        for row in rows {
            let wallet_address = WalletAddress::new(row.wallet_address).map_err(|e|
                AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let permissions: HashSet<Permission> = HashSet::new();
            let permission_plan_set: HashSet<String> = HashSet::new();
            let wallet_metadata = WalletMetadata::from_json(row.wallet_metadata)
                .unwrap_or_default();

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address,
                is_active: row.is_active,
                permissions,
                plans: permission_plan_set,
                wallet_metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at: row.last_auth_at,
                version: 1,
            });
            users.push(wallet);
        }
        Ok(users)
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
                error!("Failed to find users by permission plan {}: {}", type_filter, e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
            })?;

        let mut users = Vec::new();
        for row in rows {
            let wallet_address = WalletAddress::new(row.wallet_address).map_err(|e|
                AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let permissions: HashSet<Permission> = HashSet::new();
            let permission_plan_set: HashSet<String> = HashSet::new();
            let wallet_metadata = WalletMetadata::from_json(row.wallet_metadata)
                .unwrap_or_default();

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address,
                is_active: row.is_active,
                permissions,
                plans: permission_plan_set,
                wallet_metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at: row.last_auth_at,
                version: 1,
            });
            users.push(wallet);
        }
        Ok(users)
    }

    async fn find_by_permission_plan(&self, permission_plan: &str) -> AppResult<Vec<WalletUser>> {
        let mut conn = self.db_pool.conn().await?;

        let rows = diesel::sql_query(            r#"
            SELECT
                wallet_address,
                is_active,
                wallet_metadata,
                created_at,
                updated_at,
                last_auth_at
            FROM wallet_users
            WHERE is_active = true AND wallet_address IN (
                SELECT wga.wallet_address
                FROM wallet_plan_assignments wga
                JOIN plans pg ON wga.plan_id = pg.id
                WHERE wga.is_active = true AND pg.is_active = true
                  AND (pg.name = $1 OR pg.slug = $1)
            )
            ORDER BY created_at DESC
            "#)
            .bind::<diesel::sql_types::Text, _>(permission_plan)
            .load::<WalletUserQueryResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find users by permission plan {}: {}", permission_plan, e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation("find_by_permission_plan")
            })?;

        let mut users = Vec::new();
        for row in rows {
            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address: WalletAddress::new(row.wallet_address).expect("Invalid wallet address"),
                is_active: row.is_active,
                permissions: HashSet::new(),
                plans: HashSet::new(),
                wallet_metadata: WalletMetadata::from_json(row.wallet_metadata).expect("Invalid wallet metadata"),
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at: row.last_auth_at,
                version: 1,
            });
            users.push(wallet);
        }
        Ok(users)
    }

    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> AppResult<WalletUserSearchResult> {
        let mut conn = self.db_pool.conn().await?;

        // Use parameterized query with IS NULL trick for optional filters
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

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        plans: HashSet::new(),
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    });
                    users.push(wallet);
                }
            }
        }

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

        Ok(result[..].first().map(|r| r.count as u64).unwrap_or(0))
    }

    async fn find_by_nft_ownership(
        &self,
        contract_address: &str,
        _token_ids: Option<&[u64]>,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>> {
        warn!("find_by_nft_ownership not yet implemented for contract {} on chain {}", contract_address, chain_id);
        Ok(Vec::new())
    }

    async fn find_by_token_balance(
        &self,
        contract_address: &str,
        _min_balance: &str,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>> {
        warn!("find_by_token_balance not yet implemented for contract {} on chain {}", contract_address, chain_id);
        Ok(Vec::new())
    }

    async fn find_by_dao_membership(
        &self,
        dao_contract: &str,
        _min_voting_power: &str,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>> {
        warn!("find_by_dao_membership not yet implemented for DAO {} on chain {}", dao_contract, chain_id);
        Ok(Vec::new())
    }

    async fn validate_web3_permissions(
        &self,
        wallet_address: &WalletAddress,
        permissions: &[Permission]
    ) -> AppResult<Vec<bool>> {
        let results = permissions.iter()
            .map(|p| p.is_manual())
            .collect();
        
        info!("Validated {} permissions for wallet {}", permissions.len(), wallet_address.as_str());
        Ok(results)
    }

    async fn cache_web3_validation(
        &self,
        wallet_address: &WalletAddress,
        permission: &Permission,
        is_valid: bool,
        cache_duration_seconds: u64
    ) -> AppResult<()> {
        info!(
            "Would cache validation result for wallet {} permission {}: {} (TTL: {}s)",
            wallet_address.as_str(),
            permission.as_str(),
            is_valid,
            cache_duration_seconds
        );
        Ok(())
    }
}

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

        let query = r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
            FROM wallet_users
        "#;

        let results = diesel::sql_query(query)
            .load::<StatsResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get wallet user statistics: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation("get_statistics")
            })?;

        let stats = results[..].first().ok_or_else(|| {
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
            SELECT
                chain_id,
                count
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

        info!("Generated Web3 analytics: {} permission types, {} chains",
              permission_type_distribution.len(), chain_distribution.len());

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

        let top_nft_contracts: Vec<(String, u64)> = nft_rows.into_iter()
            .map(|row| (row.contract_address, row.user_count as u64))
            .collect();

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

        let top_token_contracts: Vec<(String, u64)> = token_rows.into_iter()
            .map(|row| (row.contract_address, row.user_count as u64))
            .collect();

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

        let top_dao_contracts: Vec<(String, u64)> = dao_rows.into_iter()
            .map(|row| (row.contract_address, row.user_count as u64))
            .collect();

        Ok(Web3Analytics {
            top_nft_contracts,
            top_token_contracts,
            top_dao_contracts,
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

        let mut distribution = HashMap::new();
        for row in rows {
            distribution.insert(row.permission_string, row.user_count as u64);
        }

        info!("Retrieved permission distribution for {} permissions", distribution.len());
        Ok(distribution)
    }

    async fn get_activity_patterns_by_chain(
        &self,
        chain_id: u64,
        days: u32
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

        let patterns: Vec<(NaiveDate, u64)> = rows.iter()
            .filter_map(|row| {
                row.auth_date.map(|date| (date, row.daily_active_users as u64))
            })
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
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        plans: HashSet::new(),
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    });
                    users.push(wallet);
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
                progression.entry(row.wallet_address)
                    .or_default()
                    .push(row.permission_plan);
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
