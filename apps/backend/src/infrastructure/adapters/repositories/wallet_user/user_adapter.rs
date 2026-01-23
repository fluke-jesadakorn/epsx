use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use crate::schemas::primary::wallet_users;
use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, NewWalletUserDb};

use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::{WalletAddress, Permission},
    repository_ports::WalletUserRepositoryPort,
};

#[derive(Clone)]
pub struct PostgresWalletUserRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PostgresWalletUserRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserRepositoryPort for PostgresWalletUserRepositoryAdapter {
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
            .select(WalletUserDb::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                tracing::error!("Failed to find wallet user by address {}: {}", wallet_address.as_str(), e);
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

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("find_by_wallets"))?;

        let addresses_lower: Vec<String> = wallet_addresses.iter()
            .map(|w| w.as_str().to_lowercase())
            .collect();

        let db_users = wallet_users::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = ANY(ARRAY[{}])",
                addresses_lower.iter()
                    .map(|a| format!("'{}'", a.replace("'", "''")))
                    .collect::<Vec<_>>()
                    .join(",")
            )))
            .order(wallet_users::created_at.desc())
            .select(WalletUserDb::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find wallet users by addresses: {}", e);
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("save"))?;

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
                tracing::error!("Failed to save wallet user {}: {}", user.wallet_address().as_str(), e);
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

        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation(format!("save_batch({} users)", users.len())))?;

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
                    tracing::error!("Failed to save wallet user in batch {}: {}", user.wallet_address().as_str(), e);
                    AppError::database_error(e.to_string())
                        .with_component("wallet_user_repository")
                    })?;
        }

        tracing::info!("Saved batch of {} wallet users", users.len());
        Ok(())
    }

    async fn delete(&self, wallet_address: &WalletAddress) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
                .with_operation("delete"))?;

        let _rows_affected = diesel::delete(
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
            tracing::error!("Failed to delete wallet user {}: {}", wallet_address.as_str(), e);
            AppError::database_error(e.to_string())
                .with_component("wallet_user_repository")
        })?;

        tracing::info!("Deleted wallet user: {}", wallet_address.as_str());
        Ok(())
    }
    
    async fn find_eligible_for_web3_permissions(&self, _chain_id: u64) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new()) // Placeholder
    }
    
    async fn health_check(&self) -> AppResult<()> {
        Ok(())
    }
    
    async fn cleanup_expired_permissions(&self) -> AppResult<u32> {
        Ok(0)
    }
}
