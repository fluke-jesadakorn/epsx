use std::collections::HashSet;

use async_trait::async_trait;
use sqlx::{PgPool, QueryBuilder};

use crate::domain::wallet_management::{
    aggregates::{WalletMetadata, WalletUser},
    repository_ports::WalletUserRepositoryPort,
    value_objects::{WalletAddress, Permission},
};
use crate::prelude::*;
use crate::infrastructure::adapters::repositories::database_types::WalletUserDb;

#[derive(Clone)]
pub struct PostgresWalletUserRepositoryAdapter {
    db_pool: &'static PgPool,
}

impl PostgresWalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserRepositoryPort for PostgresWalletUserRepositoryAdapter {
    async fn find_by_wallet(&self, wallet_address: &WalletAddress) -> AppResult<Option<WalletUser>> {
        let row: Option<WalletUserDb> = sqlx::query_as(
            "SELECT wallet_address, is_active, tier_level, wallet_metadata, \
                    permission_plans, disable_info, plan_expires_at, current_plan_id, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users \
             WHERE LOWER(wallet_address) = LOWER($1) \
             LIMIT 1",
        )
        .bind(wallet_address.as_str())
        .fetch_optional(self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to find wallet user by address {}: {}",
                wallet_address.as_str(),
                e
            );
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("find_by_wallet({})", wallet_address.as_str()))
        })?;

        Ok(match row {
            Some(row) => {
                let wallet_addr = WalletAddress::new(row.wallet_address.clone()).map_err(|e| {
                    AppError::validation_error(format!("Invalid wallet address: {}", e))
                        .with_component("wallet_user_repository")
                })?;
                let metadata = WalletMetadata::from_json(row.wallet_metadata).map_err(|e| {
                    AppError::validation_error(format!("Invalid wallet metadata: {}", e))
                        .with_component("wallet_user_repository")
                })?;
                Some(WalletUser::load(
                    crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                        wallet_address: wallet_addr,
                        is_active: row.is_active,
                        permissions: HashSet::new(),
                        plans: HashSet::new(),
                        wallet_metadata: metadata,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        last_auth_at: row.last_auth_at,
                        version: 1,
                    },
                ))
            }
            None => None,
        })
    }

    async fn find_by_wallets(&self, wallet_addresses: &[WalletAddress]) -> AppResult<Vec<WalletUser>> {
        if wallet_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let pool: PgPool = self.db_pool.clone();

        let addresses_lower: Vec<String> = wallet_addresses
            .iter()
            .map(|w| w.as_str().to_lowercase())
            .collect();

        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT wallet_address, is_active, tier_level, wallet_metadata, \
                    permission_plans, disable_info, plan_expires_at, current_plan_id, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users WHERE LOWER(wallet_address) = ANY(",
        );
        qb.push_bind(addresses_lower);
        qb.push(") ORDER BY created_at DESC");

        let rows: Vec<WalletUserDb> = qb
            .build_query_as()
            .fetch_all(pool.as_ref())
            .await
            .map_err(|e| {
                tracing::error!("Failed to find wallet users by addresses: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(format!("find_by_wallets({} addresses)", wallet_addresses.len()))
            })?;

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address.clone()) {
                let permission_set: HashSet<Permission> = HashSet::new();
                let permission_plan_set: HashSet<String> = HashSet::new();

                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata.clone()) {
                    users.push(WalletUser::load(
                        crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams {
                            wallet_address: wallet_addr,
                            is_active: row.is_active,
                            permissions: permission_set,
                            plans: permission_plan_set,
                            wallet_metadata: metadata,
                            created_at: row.created_at,
                            updated_at: row.updated_at,
                            last_auth_at: row.last_auth_at,
                            version: 1,
                        },
                    ));
                }
            }
        }
        Ok(users)
    }

    async fn save(&self, user: &WalletUser) -> AppResult<()> {
        let pool: PgPool = self.db_pool.clone();
        let metadata_json = user.wallet_metadata().to_json().map_err(|e| {
            AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                .with_component("wallet_user_repository")
        })?;

        sqlx::query(
            "INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata, updated_at, last_auth_at) \
             VALUES ($1, $2, 'free', $3, $4, $5) \
             ON CONFLICT (wallet_address) DO UPDATE SET \
                is_active = EXCLUDED.is_active, \
                wallet_metadata = EXCLUDED.wallet_metadata, \
                updated_at = EXCLUDED.updated_at, \
                last_auth_at = EXCLUDED.last_auth_at",
        )
        .bind(user.wallet_address().as_str().to_lowercase())
        .bind(user.is_active())
        .bind(&metadata_json)
        .bind(user.updated_at())
        .bind(user.last_auth_at())
        .execute(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to save wallet user {}: {}",
                user.wallet_address().as_str(),
                e
            );
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("save({})", user.wallet_address().as_str()))
        })?;

        tracing::info!("Saved wallet user: {}", user.wallet_address().as_str());
        Ok(())
    }

    async fn save_batch(&self, users: &[WalletUser]) -> AppResult<()> {
        if users.is_empty() {
            return Ok(());
        }

        let pool: PgPool = self.db_pool.clone();
        for user in users {
            let metadata_json = user.wallet_metadata().to_json().map_err(|e| {
                AppError::validation_error(format!("Failed to serialize wallet metadata: {}", e))
                    .with_component("wallet_user_repository")
            })?;
            sqlx::query(
                "INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata, updated_at, last_auth_at) \
                 VALUES ($1, $2, 'free', $3, $4, $5) \
                 ON CONFLICT (wallet_address) DO UPDATE SET \
                    is_active = EXCLUDED.is_active, \
                    wallet_metadata = EXCLUDED.wallet_metadata, \
                    updated_at = EXCLUDED.updated_at, \
                    last_auth_at = EXCLUDED.last_auth_at",
            )
            .bind(user.wallet_address().as_str().to_lowercase())
            .bind(user.is_active())
            .bind(&metadata_json)
            .bind(user.updated_at())
            .bind(user.last_auth_at())
            .execute(pool.as_ref())
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to save wallet user in batch {}: {}",
                    user.wallet_address().as_str(),
                    e
                );
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
            })?;
        }
        tracing::info!("Saved batch of {} wallet users", users.len());
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        let pool: PgPool = self.db_pool.clone();
        sqlx::query("DELETE FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)")
            .bind(wallet_address.as_str())
            .execute(pool.as_ref())
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to delete wallet user {}: {}",
                    wallet_address.as_str(),
                    e
                );
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
            })?;
        tracing::info!("Deleted wallet user: {}", wallet_address.as_str());
        Ok(())
    }

    async fn find_eligible_for_web3_permissions(&self, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }

    async fn cleanup_expired_permissions(&self) -> AppResult<u32> {
        Ok(0)
    }
}
