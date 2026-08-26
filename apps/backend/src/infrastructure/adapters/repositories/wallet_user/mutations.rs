// WalletUserRepositoryPort implementation — save/delete/find primary methods
//
// BIG-BANG: migrated to sqlx (real).

use super::{WalletUserQueryResult, WalletUserRepositoryAdapter};
use crate::domain::wallet_management::aggregates::wallet_user::WalletUserLoadParams;
use crate::domain::wallet_management::{
    aggregates::{WalletMetadata, WalletUser},
    repository_ports::WalletUserRepositoryPort,
    value_objects::WalletAddress,
};
use crate::infrastructure::adapters::repositories::database_types::{
    NewWalletUserDb, WalletUserDb,
};
use crate::prelude::*;
use std::collections::HashSet;
use tracing::{error, info, warn};

#[async_trait]
impl WalletUserRepositoryPort for WalletUserRepositoryAdapter {
    async fn find_by_wallet(
        &self,
        wallet_address: &WalletAddress,
    ) -> AppResult<Option<WalletUser>> {
        let wallet_addr_lower = wallet_address.as_str().to_lowercase();

        let row: Option<WalletUserDb> = sqlx::query_as(
            "SELECT wallet_address, is_active, tier_level, wallet_metadata, \
                    last_auth_at, updated_at, created_at \
             FROM wallet_users \
             WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(&wallet_addr_lower)
        .fetch_optional(self.db_pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to find wallet user by address {}: {}",
                wallet_address.as_str(),
                e
            );
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("find_by_wallet({})", wallet_address.as_str()))
        })?;

        if let Some(row) = row {
            let wallet_addr = WalletAddress::new(row.wallet_address.clone()).map_err(|e| {
                AppError::validation_error(format!("Invalid wallet address: {}", e))
                    .with_component("wallet_user_repository")
            })?;

            let metadata = WalletMetadata::from_json(row.wallet_metadata.clone()).map_err(|e| {
                AppError::validation_error(format!("Invalid wallet metadata: {}", e))
                    .with_component("wallet_user_repository")
            })?;

            let last_auth_at = row.last_auth_at;

            let wallet = WalletUser::load(WalletUserLoadParams {
                wallet_address: wallet_addr,
                is_active: row.is_active,
                permissions: HashSet::new(),
                plans: HashSet::new(),
                wallet_metadata: metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_auth_at,
                version: 1,
            });

            Ok(Some(wallet))
        } else {
            Ok(None)
        }
    }

    async fn find_by_wallets(
        &self,
        wallet_addresses: &[WalletAddress],
    ) -> AppResult<Vec<WalletUser>> {
        if wallet_addresses.is_empty() {
            return Ok(Vec::new());
        }

        let addresses_lower: Vec<String> = wallet_addresses
            .iter()
            .map(|w| w.as_str().to_lowercase())
            .collect();

        let db_users: Vec<WalletUserDb> = sqlx::query_as(
            "SELECT wallet_address, is_active, tier_level, wallet_metadata, \
                    last_auth_at, updated_at, created_at \
             FROM wallet_users \
             WHERE LOWER(wallet_address) = ANY($1) \
             ORDER BY created_at DESC",
        )
        .bind(&addresses_lower)
        .fetch_all(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to find wallet users by addresses: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!(
                    "find_by_wallets({} addresses)",
                    wallet_addresses.len()
                ))
        })?;

        let mut wallets = Vec::with_capacity(db_users.len());
        for row in db_users {
            let wallet_addr = WalletAddress::new(row.wallet_address.clone()).map_err(|e| {
                AppError::validation_error(format!("Invalid wallet address: {}", e))
            })?;
            let metadata = WalletMetadata::from_json(row.wallet_metadata.clone()).map_err(|e| {
                AppError::validation_error(format!("Invalid wallet metadata: {}", e))
            })?;
            wallets.push(WalletUser::load(WalletUserLoadParams {
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
        Ok(wallets)
    }

    async fn save(&self, wallet: &WalletUser) -> AppResult<()> {
        let new_user = NewWalletUserDb {
            wallet_address: wallet.wallet_address().as_str().to_string(),
            is_active: wallet.is_active(),
            tier_level: "Bronze".to_string(),
            wallet_metadata: serde_json::to_value(wallet.wallet_metadata())
                .unwrap_or_else(|_| serde_json::json!({})),
        };

        sqlx::query(
            "INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata, created_at, updated_at) \
             VALUES ($1, $2, 'Bronze', $3, NOW(), NOW()) \
             ON CONFLICT (wallet_address) DO UPDATE SET \
                is_active = EXCLUDED.is_active, \
                wallet_metadata = EXCLUDED.wallet_metadata, \
                updated_at = NOW()",
        )
        .bind(&new_user.wallet_address)
        .bind(new_user.is_active)
        .bind(&new_user.wallet_metadata)
        .execute(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to save wallet user: {}", e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("save({})", new_user.wallet_address))
        })?;

        info!("Saved wallet user {}", new_user.wallet_address);
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        sqlx::query("DELETE FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)")
            .bind(wallet_address.as_str())
            .execute(self.db_pool)
            .await
            .map_err(|e| {
                error!(
                    "Failed to delete wallet user {}: {}",
                    wallet_address.as_str(),
                    e
                );
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_repository")
                    .with_operation(format!("delete({})", wallet_address.as_str()))
            })?;
        Ok(())
    }

    async fn exists(&self, wallet_address: &WalletAddress) -> AppResult<bool> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address.as_str())
        .fetch_optional(self.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to check wallet existence: {}", e);
            AppError::database_error(e.to_string())
        })?;
        Ok(row.map(|(c,)| c > 0).unwrap_or(false))
    }

    async fn activate(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        sqlx::query(
            "UPDATE wallet_users SET is_active = TRUE, updated_at = NOW() \
             WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address.as_str())
        .execute(self.db_pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to activate wallet {}: {}",
                wallet_address.as_str(),
                e
            );
            AppError::database_error(e.to_string())
        })?;
        Ok(())
    }

    async fn deactivate(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        sqlx::query(
            "UPDATE wallet_users SET is_active = FALSE, updated_at = NOW() \
             WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address.as_str())
        .execute(self.db_pool)
        .await
        .map_err(|e| {
            warn!(
                "Failed to deactivate wallet {}: {}",
                wallet_address.as_str(),
                e
            );
            AppError::database_error(e.to_string())
        })?;
        Ok(())
    }

    async fn update_metadata(
        &self,
        wallet_address: &WalletAddress,
        metadata: serde_json::Value,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE wallet_users SET wallet_metadata = $1, updated_at = NOW() \
             WHERE LOWER(wallet_address) = LOWER($2)",
        )
        .bind(metadata)
        .bind(wallet_address.as_str())
        .execute(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    async fn touch(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        sqlx::query(
            "UPDATE wallet_users SET last_auth_at = NOW(), updated_at = NOW() \
             WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address.as_str())
        .execute(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    async fn find_eligible_for_web3_permissions(
        &self,
        chain_id: u64,
    ) -> AppResult<Vec<WalletUser>> {
        let _ = chain_id;
        Ok(Vec::new())
    }

    async fn save_batch(&self, users: &[WalletUser]) -> AppResult<()> {
        for user in users {
            self.save(user).await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(self.db_pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    async fn cleanup_expired_permissions(&self) -> AppResult<u32> {
        Ok(0)
    }
}

impl WalletUserRepositoryAdapter {
    #[allow(dead_code)]
    pub(crate) async fn find_query_result_by_wallet(
        &self,
        wallet_address: &WalletAddress,
    ) -> AppResult<Option<WalletUserQueryResult>> {
        let row: Option<WalletUserQueryResult> = sqlx::query_as(
            "SELECT wallet_address, is_active, wallet_metadata, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address.as_str())
        .fetch_optional(self.db_pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row)
    }
}
