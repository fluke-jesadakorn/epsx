// Wallet User Repository Adapter (Infrastructure Layer)
// Implements WalletUserRepositoryPort using Diesel and PostgreSQL for Web3-first authentication

use crate::prelude::*;

use std::collections::{HashMap, HashSet};
use chrono::NaiveDate;
use tracing::{error, info, warn};

// Diesel imports
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use crate::schema::wallet_users;
use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, NewWalletUserDb};

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
    db_pool: &'static Pool<AsyncPgConnection>,
}

impl WalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserRepositoryPort for WalletUserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>> {
        use diesel::dsl::*;

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_wallet"))?;

        // Diesel query: case-insensitive wallet address lookup
        let db_user = wallet_users::table
            .filter(sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = LOWER('{}')",
                wallet_address.as_str()
            )))
            .first::<WalletUserDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find wallet user by address {}: {}", wallet_address.as_str(), e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(&format!("find_by_wallet({})", wallet_address.as_str()))
            })?;

        if let Some(row) = db_user {
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
            let metadata = WalletMetadata::from_json(row.wallet_metadata)
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

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_wallets"))?;

        let addresses_lower: Vec<String> = wallet_addresses.iter()
            .map(|w| w.as_str().to_lowercase())
            .collect();

        // Diesel query: find wallets with case-insensitive matching
        let db_users = wallet_users::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = ANY(ARRAY[{}])",
                addresses_lower.iter()
                    .map(|a| format!("'{}'", a.replace("'", "''")))
                    .collect::<Vec<_>>()
                    .join(",")
            )))
            .order(wallet_users::created_at.desc())
            .load::<WalletUserDb>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find wallet users by addresses: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(&format!("find_by_wallets({} addresses)", wallet_addresses.len()))
            })?;

        let mut users = Vec::new();
        for row in db_users {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                // NOTE: Permissions now in normalized tables
                let permission_set: HashSet<Permission> = HashSet::new();
                let permission_group_set: HashSet<String> = HashSet::new();

                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
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

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("save"))?;

        // Serialize wallet metadata to JSON
        let metadata_json = user.wallet_metadata().to_json()
            .map_err(|e| AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository"))?;

        // Create new record for insert
        let new_user = NewWalletUserDb {
            wallet_address: user.wallet_address().as_str().to_lowercase(),
            is_active: user.is_active(),
            tier_level: "free".to_string(), // Default tier
            wallet_metadata: metadata_json.clone(),
        };

        // Diesel upsert: insert or update on conflict
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
                    .with_operation(&format!("save({})", user.wallet_address().as_str()))
            })?;

        info!("Saved wallet user: {}", user.wallet_address().as_str());
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("delete"))?;

        // Diesel delete: case-insensitive wallet address match
        let rows_affected = diesel::delete(
            wallet_users::table.filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "LOWER(wallet_address) = LOWER('{}')",
                    wallet_address.as_str().replace("'", "''")
                ))
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

    async fn find_by_permission(&self, permission: &Permission) -> AppResult<Vec<WalletUser>> {
        let permission_str = permission.as_str();
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_permission"))?;

        // Query users with this permission from normalized tables
        let query = format!(
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
                (p1.permission_string = '{}' AND p1.is_active = true AND wga.is_active = true)
                OR
                (p2.permission_string = '{}' AND p2.is_active = true AND wdp.is_active = true)
              )
            "#,
            permission_str.replace("'", "''"),
            permission_str.replace("'", "''")
        );

        let rows = diesel::sql_query(query)
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
        let type_filter = match permission_type {
            PermissionType::Manual => "manual",
            PermissionType::NftGated { .. } => "nft_gated",
            PermissionType::TokenGated { .. } => "token_gated",
            PermissionType::DaoGovernance { .. } => "dao_governance",
        };

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_permission_type"))?;

        let query = format!(
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
              AND (pg.name = '{}' OR pg.slug = '{}')
            ORDER BY wu.created_at DESC
            "#,
            type_filter.replace("'", "''"),
            type_filter.replace("'", "''")
        );

        let rows = diesel::sql_query(query)
            .load::<WalletUserQueryResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find users by permission group {}: {}", type_filter, e);
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

    async fn find_by_permission_group(&self, permission_group: &str) -> AppResult<Vec<WalletUser>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_permission_group"))?;

        // Use raw SQL to find users who belong to a specific permission group
        let query = format!(r#"
            SELECT DISTINCT
                wu.wallet_address,
                wu.is_active,
                wu.tier_level,
                wu.wallet_metadata,
                wu.created_at,
                wu.updated_at,
                wu.last_auth_at
            FROM wallet_users wu
            INNER JOIN wallet_group_memberships wga ON wu.wallet_address = wga.wallet_address
            INNER JOIN permission_groups pg ON wga.group_id = pg.id
            WHERE wu.is_active = true
              AND wga.is_active = true
              AND pg.is_active = true
              AND (pg.name = '{}' OR pg.slug = '{}')
            ORDER BY wu.created_at DESC
            "#,
            permission_group.replace("'", "''"),
            permission_group.replace("'", "''")
        );

        let rows = diesel::sql_query(query)
            .load::<WalletUserQueryResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find users by permission group {}: {}", permission_group, e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation("find_by_permission_group")
            })?;

        let mut users = Vec::new();
        for row in rows {
            let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                wallet_address: WalletAddress::new(row.wallet_address).expect("Invalid wallet address"),
                is_active: row.is_active,
                permissions: HashSet::new(),
                permission_groups: HashSet::new(),
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_criteria"))?;

        // Build dynamic SQL query
        let mut where_clauses = vec!["1=1".to_string()];

        if let Some(ref pattern) = criteria.wallet_pattern {
            where_clauses.push(format!("wallet_address ILIKE '%{}%'", pattern.replace("'", "''")));
        }

        if let Some(is_active) = criteria.is_active {
            where_clauses.push(format!("is_active = {}", is_active));
        }

        if let Some(ref permission_group) = criteria.permission_group {
            where_clauses.push(format!("permission_groups::jsonb ? '{}'", permission_group.replace("'", "''")));
        }

        if let Some(ref created_after) = criteria.created_after {
            where_clauses.push(format!("created_at > '{}'", created_after.to_rfc3339()));
        }

        if let Some(ref created_before) = criteria.created_before {
            where_clauses.push(format!("created_at < '{}'", created_before.to_rfc3339()));
        }

        let query = format!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE {}
            ORDER BY created_at DESC
            LIMIT {}
            OFFSET {}
            "#,
            where_clauses.join(" AND "),
            limit,
            offset
        );

        let rows = diesel::sql_query(query)
            .load::<WalletUserQueryResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to search wallet users: {}", e);
                AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
            })?;

        // Convert rows to users
        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    let wallet = WalletUser::load(crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        permission_groups: HashSet::new(),
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

        // Get total count
        let total_count = self.count_by_criteria(criteria).await?;

        Ok(WalletUserSearchResult::new(users, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("count_by_criteria"))?;

        // Build dynamic SQL query
        let mut where_clauses = vec!["1=1".to_string()];

        if let Some(ref pattern) = criteria.wallet_pattern {
            where_clauses.push(format!("wallet_address ILIKE '%{}%'", pattern.replace("'", "''")));
        }

        if let Some(is_active) = criteria.is_active {
            where_clauses.push(format!("is_active = {}", is_active));
        }

        let query = format!(
            "SELECT COUNT(*) as count FROM wallet_users WHERE {}",
            where_clauses.join(" AND ")
        );

        #[derive(diesel::QueryableByName)]
        struct CountResult {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let result = diesel::sql_query(query)
            .load::<CountResult>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count wallet users: {}", e);
                AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
            })?;

        Ok(result.get(0).map(|r| r.count as u64).unwrap_or(0))
    }

    async fn find_eligible_for_web3_permissions(&self, chain_id: u64) -> AppResult<Vec<WalletUser>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_eligible_for_web3_permissions"))?;

        let query = format!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (wallet_metadata->>'primary_chain_id')::bigint = {}
            "#,
            chain_id
        );

        let rows = diesel::sql_query(query)
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
                        permission_groups: HashSet::new(),
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

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(&format!("save_batch({} users)", users.len())))?;

        // Execute batch inserts without transaction for now
        // Transaction support can be added later if needed for data consistency
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("health_check"))?;

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

#[async_trait]
impl WalletUserAnalyticsPort for WalletUserRepositoryAdapter {
    async fn get_statistics(&self) -> AppResult<WalletUserStatistics> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_statistics"))?;

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

        let stats = results.get(0).ok_or_else(|| {
            AppError::database_error("No statistics returned".to_string())
                .with_component("wallet_user_repository")
        })?;

        // Tier system removed - permission groups tracked via wallet_user_groups table
        let users_by_permission_group = HashMap::new();

        Ok(WalletUserStatistics {
            total_users: stats.total_users as u64,
            active_users: stats.active_users as u64,
            users_by_permission_group,
            users_by_chain: HashMap::new(), // Chain distribution to be implemented when needed
            manual_permissions: 0, // Manual permissions count to be implemented
            nft_gated_permissions: 0, // NFT-gated permissions count to be implemented
            token_gated_permissions: 0, // Token-gated permissions count to be implemented
            dao_governance_permissions: 0, // DAO governance permissions count to be implemented
            recent_authentications_24h: stats.recent_auth_24h as u64,
            new_wallets_24h: stats.new_wallets_24h as u64,
        })
    }

    async fn get_web3_analytics(&self) -> AppResult<Web3Analytics> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_web3_analytics"))?;

        // Query permission type distribution
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

        // Query chain distribution from wallet metadata
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
                (wallet_metadata->>'primary_chain_id')::text as chain_id,
                COUNT(*) as count
            FROM wallet_users
            WHERE is_active = true
              AND wallet_metadata->>'primary_chain_id' IS NOT NULL
            GROUP BY (wallet_metadata->>'primary_chain_id')::text
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

        Ok(Web3Analytics {
            top_nft_contracts: Vec::new(), // TODO: Requires blockchain indexer integration
            top_token_contracts: Vec::new(), // TODO: Requires blockchain indexer integration
            top_dao_contracts: Vec::new(), // TODO: Requires blockchain indexer integration
            chain_distribution,
            permission_type_distribution,
        })
    }

    async fn get_permission_distribution(&self) -> AppResult<HashMap<String, u64>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_permission_distribution"))?;

        // Query permission usage across all active users
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
            LEFT JOIN permission_group_memberships pgm ON p.id = pgm.permission_id
            LEFT JOIN wallet_group_memberships wgm ON pgm.group_id = wgm.group_id AND wgm.is_active = true
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(&format!("get_activity_patterns_by_chain(chain={}, days={})", chain_id, days)))?;

        #[derive(diesel::QueryableByName)]
        struct ActivityRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Date>)]
            auth_date: Option<NaiveDate>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            daily_active_users: i64,
        }

        let query = format!(
            r#"
            SELECT
                DATE(last_auth_at) as auth_date,
                COUNT(DISTINCT wallet_address) as daily_active_users
            FROM wallet_users
            WHERE is_active = true
              AND last_auth_at >= '{}'
              AND (wallet_metadata->>'primary_chain_id')::bigint = {}
            GROUP BY DATE(last_auth_at)
            ORDER BY auth_date DESC
            "#,
            cutoff_date.to_rfc3339(),
            chain_id
        );

        let rows = diesel::sql_query(query)
            .load::<ActivityRow>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to get activity patterns for chain {}: {}", chain_id, e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(&format!("get_activity_patterns_by_chain(chain={}, days={})", chain_id, days))
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_inactive_users"))?;

        let query = format!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            AND (last_auth_at IS NULL OR last_auth_at < '{}')
            ORDER BY last_auth_at ASC NULLS FIRST
            "#,
            cutoff_date.to_rfc3339()
        );

        let rows = diesel::sql_query(query)
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
                        permission_groups: HashSet::new(),
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

    async fn get_group_progression(&self) -> AppResult<HashMap<String, Vec<String>>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("get_group_progression"))?;

        // Query historical group assignments from audit log
        #[derive(diesel::QueryableByName)]
        struct ProgressionRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            wallet_address: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            permission_group: String,
        }

        let rows = diesel::sql_query(
            r#"
            SELECT DISTINCT ON (wallet_address, permission_group)
                wallet_address,
                permission_group,
                event_timestamp
            FROM permission_audit_log
            WHERE event_type = 'group_assigned'
              AND permission_group IS NOT NULL
            ORDER BY wallet_address, permission_group, event_timestamp ASC
            "#
        )
        .load::<ProgressionRow>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to get group progression: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        let mut progression: HashMap<String, Vec<String>> = HashMap::new();
        for row in rows {
            if !row.wallet_address.is_empty() && !row.permission_group.is_empty() {
                progression.entry(row.wallet_address)
                    .or_insert_with(Vec::new)
                    .push(row.permission_group);
            }
        }

        info!("Retrieved group progression for {} wallets", progression.len());
        Ok(progression)
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

// Include the comprehensive test modules
#[cfg(test)]
mod wallet_user_repository_test;
#[cfg(test)]
mod transaction_tests;