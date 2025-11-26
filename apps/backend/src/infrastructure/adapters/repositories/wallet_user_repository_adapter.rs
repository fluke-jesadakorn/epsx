// Wallet User Repository Adapter (Infrastructure Layer)
// Implements WalletUserRepositoryPort using SQLx and PostgreSQL for Web3-first authentication

use crate::prelude::*;

use std::collections::{HashMap, HashSet};
use chrono::NaiveDate;
use sqlx::{PgPool, Row};
use tracing::{error, info, warn};

use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::{WalletAddress, Permission, PermissionType},
    repository_ports::{
        WalletUserRepositoryPort,
        WalletUserAnalyticsPort,
        WalletUserSearchCriteria,
        WalletUserSearchResult,
        WalletUserStatistics,
        Web3Analytics,
    },
};

/// PostgreSQL implementation of WalletUserRepositoryPort
#[derive(Clone)]
pub struct WalletUserRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl WalletUserRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserRepositoryPort for WalletUserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>> {
        let row = sqlx::query!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE LOWER(wallet_address) = LOWER($1)
            "#,
            wallet_address.as_str()
        )
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find wallet user by address {}: {}", wallet_address.as_str(), e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        if let Some(row) = row {
            let wallet_addr = WalletAddress::new(row.wallet_address)
                .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e))
                    .with_component("wallet_user_repository"))?;

            // NOTE: Permissions now stored in normalized tables
            // Use Web3PermissionService or query normalized tables directly:
            // - wallet_group_memberships + permission_group_memberships (group permissions)
            // - wallet_direct_permissions (direct permissions)
            let permission_set: HashSet<Permission> = HashSet::new();
            let permission_group_set: HashSet<String> = HashSet::new();

            // Parse wallet metadata
            let metadata_json = row.wallet_metadata;
            let metadata = WalletMetadata::from_json(metadata_json)
                .map_err(|e| AppError::validation_error(format!("Invalid wallet metadata: {}", e))
                    .with_component("wallet_user_repository"))?;

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address: wallet_addr,
                is_active: row.is_active,
                permissions: permission_set,
                permission_groups: permission_group_set,
                wallet_metadata: metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at: row.last_auth_at,
                version: 1, // Version - TODO: implement proper versioning
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

        let addresses: Vec<String> = wallet_addresses.iter().map(|w| w.as_str().to_string()).collect();
        
        let rows = sqlx::query!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE LOWER(wallet_address) = ANY(SELECT LOWER(unnest($1::text[])))
            ORDER BY created_at DESC
            "#,
            &addresses
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find wallet users by addresses: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                // NOTE: Permissions now in normalized tables
                let permission_set: HashSet<Permission> = HashSet::new();
                let permission_group_set: HashSet<String> = HashSet::new();

                let metadata_json = row.wallet_metadata;
                if let Ok(metadata) = WalletMetadata::from_json(metadata_json) {
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: permission_set,
                        permission_groups: permission_group_set,
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
        // NOTE: Permissions now stored in normalized tables
        // Use separate APIs to manage permissions:
        // - wallet_group_memberships (assign wallet to groups)
        // - wallet_direct_permissions (grant direct permissions)

        // Serialize wallet metadata to JSON
        let metadata_json = user.wallet_metadata().to_json()
            .map_err(|e| AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository"))?;

        sqlx::query!(
            r#"
            INSERT INTO wallet_users (
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (wallet_address)
            DO UPDATE SET
                is_active = EXCLUDED.is_active,
                wallet_metadata = EXCLUDED.wallet_metadata,
                updated_at = EXCLUDED.updated_at,
                last_auth_at = EXCLUDED.last_auth_at
            "#,
            user.wallet_address().as_str().to_lowercase(),
            user.is_active(),
            metadata_json,
            user.created_at(),
            user.updated_at(),
            user.last_auth_at()
        )
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to save wallet user {}: {}", user.wallet_address().as_str(), e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        info!("Saved wallet user: {}", user.wallet_address().as_str());
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        let result = sqlx::query!(
            "DELETE FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)",
            wallet_address.as_str()
        )
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to delete wallet user {}: {}", wallet_address.as_str(), e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        if result.rows_affected() > 0 {
            info!("Deleted wallet user: {}", wallet_address.as_str());
        } else {
            warn!("No wallet user found to delete: {}", wallet_address.as_str());
        }

        Ok(())
    }

    async fn find_by_permission(&self, permission: &Permission) -> AppResult<Vec<WalletUser>> {
        let permission_str = permission.as_str();

        // Query users with this permission from normalized tables
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT
                wu.wallet_address, wu.is_active, wu.wallet_metadata,
                wu.created_at, wu.updated_at, wu.last_auth_at
            FROM wallet_users wu
            LEFT JOIN wallet_group_memberships wga ON wu.wallet_address = wga.wallet_address
            LEFT JOIN permission_group_memberships pgm ON wga.group_id = pgm.group_id
            LEFT JOIN permissions p1 ON pgm.permission_id = p1.id
            LEFT JOIN wallet_direct_permissions wdp ON wu.wallet_address = wdp.wallet_address
            LEFT JOIN permissions p2 ON wdp.permission_id = p2.id
            WHERE wu.is_active = true
              AND (
                (p1.permission_string = $1 AND p1.is_active = true AND wga.is_active = true)
                OR
                (p2.permission_string = $1 AND p2.is_active = true AND wdp.is_active = true)
              )
            "#,
            permission_str
        )
        .fetch_all(self.db_pool.as_ref())
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
            let permission_group_set: HashSet<String> = HashSet::new();
            let wallet_metadata = WalletMetadata::from_json(row.wallet_metadata)
                .unwrap_or_default();

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address,
                is_active: row.is_active,
                permissions,
                permission_groups: permission_group_set,
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
        // This would require storing permission type metadata in the database
        // For now, return empty result with a TODO
        warn!("find_by_permission_type not yet implemented for permission type: {:?}", permission_type);
        Ok(Vec::new())
    }

    async fn find_by_permission_group(&self, permission_group: &str) -> AppResult<Vec<WalletUser>> {
        // Query users assigned to this group from normalized tables
        // Note: permission_group can be group name or group slug
        let rows = sqlx::query!(
            r#"
            SELECT DISTINCT
                wu.wallet_address, wu.is_active, wu.wallet_metadata,
                wu.created_at, wu.updated_at, wu.last_auth_at
            FROM wallet_users wu
            INNER JOIN wallet_group_memberships wga ON wu.wallet_address = wga.wallet_address
            INNER JOIN permission_groups pg ON wga.group_id = pg.id
            WHERE wu.is_active = true
              AND wga.is_active = true
              AND pg.is_active = true
              AND (pg.name = $1 OR pg.slug = $1)
            ORDER BY wu.created_at DESC
            "#,
            permission_group
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find users by permission group {}: {}", permission_group, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let mut users = Vec::new();
        for row in rows {
            let wallet_address = WalletAddress::new(row.wallet_address).map_err(|e|
                AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

            let permissions: HashSet<Permission> = HashSet::new();
            let permission_group_set: HashSet<String> = HashSet::new();
            let wallet_metadata = WalletMetadata::from_json(row.wallet_metadata)
                .unwrap_or_default();

            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address,
                is_active: row.is_active,
                permissions,
                permission_groups: permission_group_set,
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

    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> AppResult<WalletUserSearchResult> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT
                wallet_address, is_active, permissions, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE 1=1
            "#
        );

        // Apply search criteria
        if let Some(ref pattern) = criteria.wallet_pattern {
            query_builder.push(" AND wallet_address ILIKE ");
            query_builder.push_bind(format!("%{}%", pattern));
        }

        if let Some(is_active) = criteria.is_active {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(is_active);
        }

        if let Some(ref permission_group) = criteria.permission_group {
            query_builder.push(" AND permission_groups::jsonb ? ");
            query_builder.push_bind(permission_group);
        }

        if let Some(ref created_after) = criteria.created_after {
            query_builder.push(" AND created_at > ");
            query_builder.push_bind(created_after);
        }

        if let Some(ref created_before) = criteria.created_before {
            query_builder.push(" AND created_at < ");
            query_builder.push_bind(created_before);
        }

        // Add ordering and pagination
        query_builder.push(" ORDER BY created_at DESC LIMIT ");
        query_builder.push_bind(limit as i64);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset as i64);

        let query = query_builder.build();
        let rows = query.fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to search wallet users: {}", e);
                AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
            })?;

        // Convert rows to users - simplified version
        let users = self.rows_to_wallet_users_simple(rows).await?;

        // Get total count (simplified - in production, you'd want to optimize this)
        let total_count = self.count_by_criteria(criteria).await?;

        Ok(WalletUserSearchResult::new(users, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM wallet_users WHERE 1=1");

        // Apply same criteria as search (simplified)
        if let Some(ref pattern) = criteria.wallet_pattern {
            query_builder.push(" AND wallet_address ILIKE ");
            query_builder.push_bind(format!("%{}%", pattern));
        }

        if let Some(is_active) = criteria.is_active {
            query_builder.push(" AND is_active = ");
            query_builder.push_bind(is_active);
        }

        let query = query_builder.build_query_scalar::<i64>();
        let count = query.fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to count wallet users: {}", e);
                AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
            })?;

        Ok(count as u64)
    }

    async fn find_eligible_for_web3_permissions(&self, chain_id: u64) -> AppResult<Vec<WalletUser>> {
        // Find active users who don't have Web3 permissions for this chain yet
        let rows = sqlx::query(
            r#"
            SELECT
                wallet_address, is_active, permissions, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (wallet_metadata->>'primary_chain_id')::bigint = $1
            "#,
        )
        .bind(chain_id as i64)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find eligible users for chain {}: {}", chain_id, e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        self.rows_to_users(rows).await
    }

    async fn save_batch(&self, users: &[WalletUser]) -> AppResult<()> {
        if users.is_empty() {
            return Ok(());
        }

        let mut tx = self.db_pool.begin().await
            .map_err(|e| AppError::database_error(format!("Transaction error: {}", e))
                .with_component("wallet_user_repository"))?;

        for user in users {
            // NOTE: Permissions now in normalized tables, managed separately
            let metadata_json = user.wallet_metadata().to_json()
                .map_err(|e| AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository"))?;

            sqlx::query!(
                r#"
                INSERT INTO wallet_users (
                    wallet_address, is_active, wallet_metadata,
                    created_at, updated_at, last_auth_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (wallet_address)
                DO UPDATE SET
                    is_active = EXCLUDED.is_active,
                    wallet_metadata = EXCLUDED.wallet_metadata,
                    updated_at = EXCLUDED.updated_at,
                    last_auth_at = EXCLUDED.last_auth_at
                "#,
                user.wallet_address().as_str().to_lowercase(),
                user.is_active(),
                metadata_json,
                user.created_at(),
                user.updated_at(),
                user.last_auth_at()
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to save wallet user in batch {}: {}", user.wallet_address().as_str(), e);
                AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
            })?;
        }

        tx.commit().await
            .map_err(|e| AppError::database_error(format!("Transaction commit error: {}", e))
                .with_component("wallet_user_repository"))?;

        info!("Saved batch of {} wallet users", users.len());
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        sqlx::query!("SELECT 1 as health_check")
            .fetch_one(self.db_pool.as_ref())
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
        // This would require parsing all permissions and removing expired ones
        // For now, return 0 as cleanup count
        warn!("cleanup_expired_permissions not yet implemented");
        Ok(0)
    }

    // Web3-specific methods
    async fn find_by_nft_ownership(
        &self,
        contract_address: &str,
        _token_ids: Option<&[u64]>,
        chain_id: u64
    ) -> AppResult<Vec<WalletUser>> {
        // This would require integration with blockchain indexing services
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
        // For now, return true for all manual permissions, false for Web3 permissions (not implemented)
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
        // This would integrate with Redis or another cache
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

// Helper methods
impl WalletUserRepositoryAdapter {
    async fn rows_to_users(&self, rows: Vec<sqlx::postgres::PgRow>) -> AppResult<Vec<WalletUser>> {
        let mut users = Vec::new();
        
        for row in rows {
            let wallet_address: String = row.get("wallet_address");
            let is_active: bool = row.get("is_active");
            let permissions_json: serde_json::Value = row.get("permissions");
            let wallet_metadata: serde_json::Value = row.get("wallet_metadata");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");
            let last_auth_at: Option<DateTime<Utc>> = row.get("last_auth_at");

            if let Ok(wallet_addr) = WalletAddress::new(wallet_address) {
                if let Ok(permission_strings) = serde_json::from_value::<Vec<String>>(permissions_json) {
                    let permission_set: HashSet<Permission> = permission_strings
                        .into_iter()
                        .filter_map(|p| Permission::new(p).ok())
                        .collect();

                    // Permission groups are now loaded from wallet_user_groups relationship table
                    // Default to empty set - groups loaded separately if needed
                    let permission_groups = HashSet::new();

                    if let Ok(metadata) = WalletMetadata::from_json(wallet_metadata) {
                        let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                            wallet_address: wallet_addr,
                            is_active,
                            permissions: permission_set,
                            permission_groups,
                            wallet_metadata: metadata,
                            created_at,
                            updated_at,
                            last_auth_at,
                            version: 1,
                        });
                        users.push(wallet);
                    }
                }
            }
        }

        Ok(users)
    }

    async fn rows_to_wallet_users_simple(&self, rows: Vec<sqlx::postgres::PgRow>) -> AppResult<Vec<WalletUser>> {
        // Simplified version that doesn't fail on individual row parsing errors
        let mut users = Vec::new();
        
        for row in rows {
            if let (Ok(wallet_address), Ok(is_active)) = (
                row.try_get::<String, _>("wallet_address"),
                row.try_get::<bool, _>("is_active")
            ) {
                if let Ok(wallet_addr) = WalletAddress::new(wallet_address) {
                    // Use defaults if parsing fails
                    let permissions = HashSet::new(); // Simplified
                    let permission_groups = HashSet::from(["Basic Access Group".to_string()]);
                    let metadata = WalletMetadata::default();
                    let created_at = row.try_get("created_at").unwrap_or_else(|_| Utc::now());
                    let updated_at = row.try_get("updated_at").unwrap_or_else(|_| Utc::now());
                    let last_auth_at = row.try_get("last_auth_at").ok();

                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active,
                        permissions,
                        permission_groups,
                        wallet_metadata: metadata,
                        created_at,
                        updated_at,
                        last_auth_at,
                        version: 1,
                    });
                    users.push(wallet);
                }
            }
        }

        Ok(users)
    }
}

#[async_trait]
impl WalletUserAnalyticsPort for WalletUserRepositoryAdapter {
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_users,
                COUNT(*) FILTER (WHERE is_active = true) as active_users,
                COUNT(*) FILTER (WHERE last_auth_at > NOW() - INTERVAL '24 hours') as recent_auth_24h,
                COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') as new_wallets_24h
            FROM wallet_users
            "#
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to get wallet user statistics: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        // Tier system removed - permission groups tracked via wallet_user_groups table
        let users_by_permission_group = HashMap::new();

        Ok(WalletUserStatistics {
            total_users: stats.total_users.unwrap_or(0) as u64,
            active_users: stats.active_users.unwrap_or(0) as u64,
            users_by_permission_group,
            users_by_chain: HashMap::new(), // TODO: implement
            manual_permissions: 0, // TODO: implement
            nft_gated_permissions: 0, // TODO: implement
            token_gated_permissions: 0, // TODO: implement
            dao_governance_permissions: 0, // TODO: implement
            recent_authentications_24h: stats.recent_auth_24h.unwrap_or(0) as u64,
            new_wallets_24h: stats.new_wallets_24h.unwrap_or(0) as u64,
        })
    }

    async fn get_web3_analytics(&self) -> AppResult<Web3Analytics> {
        // Placeholder implementation
        Ok(Web3Analytics {
            top_nft_contracts: Vec::new(),
            top_token_contracts: Vec::new(),
            top_dao_contracts: Vec::new(),
            chain_distribution: HashMap::new(),
            permission_type_distribution: HashMap::new(),
        })
    }

    async fn get_permission_distribution(&self) -> AppResult<HashMap<String, u64>> {
        // Placeholder implementation
        Ok(HashMap::new())
    }

    async fn get_activity_patterns_by_chain(
        &self,
        _chain_id: u64,
        _days: u32
    ) -> AppResult<Vec<(NaiveDate, u64)>> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    async fn find_inactive_users(&self, days: u32) -> AppResult<Vec<WalletUser>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);

        let rows = sqlx::query(
            r#"
            SELECT
                wallet_address, is_active, permissions, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (last_auth_at IS NULL OR last_auth_at < $1)
            ORDER BY last_auth_at ASC NULLS FIRST
            "#,
        )
        .bind(cutoff_date)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find inactive users: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        self.rows_to_users(rows).await
    }

    async fn get_group_progression(&self) -> AppResult<HashMap<String, Vec<String>>> {
        // Placeholder implementation for group progression analytics
        Ok(HashMap::new())
    }

    async fn get_validation_success_rates(&self) -> AppResult<HashMap<String, f64>> {
        // Placeholder implementation for Web3 validation success rates
        Ok(HashMap::new())
    }

    async fn get_cross_chain_analysis(&self) -> AppResult<HashMap<String, u64>> {
        // Placeholder implementation for cross-chain analysis
        Ok(HashMap::new())
    }
}