use std::collections::HashSet;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::wallet_management::{
    aggregates::{WalletMetadata, WalletUser},
    repository_ports::{
        WalletUserSearchCriteria, WalletUserSearchPort, WalletUserSearchResult,
    },
    value_objects::{WalletAddress},
};
use crate::prelude::*;

pub struct PostgresWalletUserSearchAdapter {
    db_pool: &'static PgPool,
}

impl PostgresWalletUserSearchAdapter {
    pub fn new(db_pool: &'static PgPool) -> Self {
        Self { db_pool }
    }
}

#[derive(sqlx::FromRow)]
struct WalletSearchRow {
    wallet_address: String,
    is_active: bool,
    wallet_metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    last_auth_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
impl WalletUserSearchPort for PostgresWalletUserSearchAdapter {
    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32,
    ) -> AppResult<WalletUserSearchResult> {
        let mut sql = String::from(
            "SELECT wallet_address, is_active, wallet_metadata, \
                    created_at, updated_at, last_auth_at \
             FROM wallet_users WHERE TRUE",
        );
        if criteria.wallet_pattern.is_some() {
            sql.push_str(" AND wallet_address ILIKE $1");
        }
        if criteria.is_active.is_some() {
            sql.push_str(if criteria.wallet_pattern.is_some() {
                " AND is_active = $2"
            } else {
                " AND is_active = $1"
            });
        }
        sql.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            match (criteria.wallet_pattern.is_some(), criteria.is_active.is_some()) {
                (true, true) => 3,
                (true, false) | (false, true) => 2,
                (false, false) => 1,
            },
            match (criteria.wallet_pattern.is_some(), criteria.is_active.is_some()) {
                (true, true) => 4,
                (true, false) | (false, true) => 3,
                (false, false) => 2,
            },
        ));

        let search_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));

        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(&sql);
        if let Some(p) = &search_pattern {
            qb.push_bind(p);
        }
        if let Some(active) = criteria.is_active {
            qb.push_bind(active);
        }
        qb.push_bind(limit as i64);
        qb.push_bind(offset as i64);

        let rows: Vec<WalletSearchRow> = qb
            .build_query_as()
            .fetch_all(self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to search wallet users: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_search_adapter")
            })?;

        let mut users = Vec::new();
        for row in rows {
            if let Ok(wallet_addr) = WalletAddress::new(row.wallet_address) {
                if let Ok(metadata) = WalletMetadata::from_json(row.wallet_metadata) {
                    let wallet = WalletUser::load(
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
                    );
                    users.push(wallet);
                }
            }
        }

        let total_count = self.count_by_criteria(criteria).await?;
        Ok(WalletUserSearchResult::new(users, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
        let mut sql = String::from("SELECT COUNT(*)::BIGINT FROM wallet_users WHERE TRUE");
        if criteria.wallet_pattern.is_some() {
            sql.push_str(" AND wallet_address ILIKE $1");
        }
        if criteria.is_active.is_some() {
            sql.push_str(if criteria.wallet_pattern.is_some() {
                " AND is_active = $2"
            } else {
                " AND is_active = $1"
            });
        }

        let search_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));

        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(&sql);
        if let Some(p) = &search_pattern {
            qb.push_bind(p);
        }
        if let Some(active) = criteria.is_active {
            qb.push_bind(active);
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count wallet users: {}", e);
                AppError::database_error(e.to_string())
                    .with_component("wallet_user_search_adapter")
            })?;

        Ok(row.0 as u64)
    }

    async fn find_by_permission(
        &self,
        _permission: &crate::domain::wallet_management::value_objects::Permission,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_permission_type(
        &self,
        _permission_type: &crate::domain::wallet_management::value_objects::PermissionType,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_permission_plan(&self, _permission_plan: &str) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_nft_ownership(
        &self,
        _contract: &str,
        _tokens: Option<&[u64]>,
        _chain: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_token_balance(
        &self,
        _contract: &str,
        _min: &str,
        _chain: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_dao_membership(
        &self,
        _dao: &str,
        _min: &str,
        _chain: u64,
    ) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn validate_web3_permissions(
        &self,
        _addr: &WalletAddress,
        _perms: &[crate::domain::wallet_management::value_objects::Permission],
    ) -> AppResult<Vec<bool>> {
        Ok(vec![])
    }

    async fn cache_web3_validation(
        &self,
        _addr: &WalletAddress,
        _perm: &crate::domain::wallet_management::value_objects::Permission,
        _valid: bool,
        _ttl: u64,
    ) -> AppResult<()> {
        Ok(())
    }
}
