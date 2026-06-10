// WalletUserRepositoryPort implementation — save/delete/find primary methods

use crate::prelude::*;
use std::collections::HashSet;
use tracing::{error, info, warn};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::schemas::primary::wallet_users;
use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, NewWalletUserDb};
use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::WalletAddress,
    repository_ports::WalletUserRepositoryPort,
};
use super::{WalletUserRepositoryAdapter, WalletUserQueryResult, lower};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;

#[async_trait]
impl WalletUserRepositoryPort for WalletUserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>> {
        let mut conn = self.db_pool.conn().await?;
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

            let metadata = WalletMetadata::from_json(row.wallet_metadata)
                .map_err(|e| AppError::validation_error(format!("Invalid wallet metadata: {}", e))
                    .with_component("wallet_user_repository"))?;

            let wallet = WalletUser::load(WalletUserLoadParams {
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

    async fn save(&self, user: &WalletUser) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

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
            wallet_users::table.filter(lower(wallet_users::wallet_address).eq(wallet_addr_lower))
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
