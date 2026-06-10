use async_trait::async_trait;

pub struct PostgresWalletUserSearchAdapter {
    // pool: DbPool,
}

impl PostgresWalletUserSearchAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use crate::schemas::primary::wallet_users;
use crate::infrastructure::adapters::repositories::database_types::{WalletUserDb, WalletUserQueryResult};

use crate::domain::wallet_management::{
    aggregates::{WalletUser, WalletMetadata},
    value_objects::{WalletAddress, Permission, PermissionType},
    repository_ports::{WalletUserSearchPort, WalletUserSearchCriteria, WalletUserSearchResult},
};

pub struct PostgresWalletUserSearchAdapter {
    db_pool: &'static TlsPool,
}

impl PostgresWalletUserSearchAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl WalletUserSearchPort for PostgresWalletUserSearchAdapter {
    async fn find_by_criteria(
        &self,
        criteria: &WalletUserSearchCriteria,
        limit: u32,
        offset: u32
    ) -> AppResult<WalletUserSearchResult> {
        let mut conn = self.db_pool.conn().await?;

        // Build parameterized dynamic SQL query
        let mut where_clauses = vec!["1=1".to_string()];
        let mut param_idx = 1u32;

        let has_pattern = criteria.wallet_pattern.is_some();
        if has_pattern {
            where_clauses.push(format!("wallet_address ILIKE ${}", param_idx));
            param_idx += 1;
        }

        let has_active = criteria.is_active.is_some();
        if has_active {
            where_clauses.push(format!("is_active = ${}", param_idx));
            param_idx += 1;
        }

        let query = format!(
            r#"
            SELECT
                wallet_address, is_active, wallet_metadata,
                created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE {}
            ORDER BY created_at DESC
            LIMIT ${} OFFSET ${}
            "#,
            where_clauses.join(" AND "),
            param_idx,
            param_idx + 1
        );

        let search_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));

        let rows = match (has_pattern, has_active) {
            (true, true) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(search_pattern.as_ref().unwrap())
                    .bind::<diesel::sql_types::Bool, _>(criteria.is_active.unwrap())
                    .bind::<diesel::sql_types::Integer, _>(limit as i32)
                    .bind::<diesel::sql_types::Integer, _>(offset as i32)
                    .load::<WalletUserQueryResult>(&mut conn).await
            }
            (true, false) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(search_pattern.as_ref().unwrap())
                    .bind::<diesel::sql_types::Integer, _>(limit as i32)
                    .bind::<diesel::sql_types::Integer, _>(offset as i32)
                    .load::<WalletUserQueryResult>(&mut conn).await
            }
            (false, true) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Bool, _>(criteria.is_active.unwrap())
                    .bind::<diesel::sql_types::Integer, _>(limit as i32)
                    .bind::<diesel::sql_types::Integer, _>(offset as i32)
                    .load::<WalletUserQueryResult>(&mut conn).await
            }
            (false, false) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Integer, _>(limit as i32)
                    .bind::<diesel::sql_types::Integer, _>(offset as i32)
                    .load::<WalletUserQueryResult>(&mut conn).await
            }
        }
        .map_err(|e| {
            tracing::error!("Failed to search wallet users: {}", e);
            AppError::database_error(e.to_string())
            .with_component("wallet_user_search_adapter")
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

        let total_count = self.count_by_criteria(criteria).await?;

        Ok(WalletUserSearchResult::new(users, total_count, offset, limit))
    }

    async fn count_by_criteria(&self, criteria: &WalletUserSearchCriteria) -> AppResult<u64> {
         let mut conn = self.db_pool.conn().await?;

        let mut where_clauses = vec!["1=1".to_string()];
        let mut param_idx = 1u32;

        let has_pattern = criteria.wallet_pattern.is_some();
        if has_pattern {
            where_clauses.push(format!("wallet_address ILIKE ${}", param_idx));
            param_idx += 1;
        }

        let has_active = criteria.is_active.is_some();
        if has_active {
            where_clauses.push(format!("is_active = ${}", param_idx));
            let _ = param_idx; // used above
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

        let search_pattern = criteria.wallet_pattern.as_ref().map(|p| format!("%{}%", p));

        let result = match (has_pattern, has_active) {
            (true, true) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(search_pattern.as_ref().unwrap())
                    .bind::<diesel::sql_types::Bool, _>(criteria.is_active.unwrap())
                    .load::<CountResult>(&mut conn).await
            }
            (true, false) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Text, _>(search_pattern.as_ref().unwrap())
                    .load::<CountResult>(&mut conn).await
            }
            (false, true) => {
                diesel::sql_query(&query)
                    .bind::<diesel::sql_types::Bool, _>(criteria.is_active.unwrap())
                    .load::<CountResult>(&mut conn).await
            }
            (false, false) => {
                diesel::sql_query(&query)
                    .load::<CountResult>(&mut conn).await
            }
        }
        .map_err(|e| {
            tracing::error!("Failed to count wallet users: {}", e);
            AppError::database_error(e.to_string())
            .with_component("wallet_user_search_adapter")
        })?;

        Ok(result.first().map(|r| r.count as u64).unwrap_or(0))
    }

    async fn find_by_permission(&self, permission: &Permission) -> AppResult<Vec<WalletUser>> {
        // Implement logic or reuse find_by_criteria?
        // Assuming implementation similar to original file
        Ok(Vec::new())
    }
    
    async fn find_by_permission_type(&self, permission_type: &PermissionType) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }
    
    async fn find_by_permission_plan(&self, permission_plan: &str) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }

    async fn find_by_nft_ownership(&self, _contract: &str, _tokens: Option<&[u64]>, _chain: u64) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }
    
    async fn find_by_token_balance(&self, _contract: &str, _min: &str, _chain: u64) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }
    
    async fn find_by_dao_membership(&self, _dao: &str, _min: &str, _chain: u64) -> AppResult<Vec<WalletUser>> {
        Ok(Vec::new())
    }
    
    async fn validate_web3_permissions(&self, _addr: &WalletAddress, _perms: &[Permission]) -> AppResult<Vec<bool>> {
        Ok(vec![])
    }
    
    async fn cache_web3_validation(&self, _addr: &WalletAddress, _perm: &Permission, _valid: bool, _ttl: u64) -> AppResult<()> {
        Ok(())
    }
}


