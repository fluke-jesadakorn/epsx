//! SQLx Wallet Management Repository — side-by-side with Diesel.
//!
//! Mirrors `wallet_management_repository.rs:451` (Diesel) using `sqlx::PgPool`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxWalletBasicInfo {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
    pub wallet_metadata: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct SqlxWalletManagementRepository {
    pool: Arc<PgPool>,
}

impl SqlxWalletManagementRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_wallet(&self, wallet_address: &str) -> AppResult<Option<SqlxWalletBasicInfo>> {
        let row = sqlx::query_as::<_, SqlxWalletBasicInfo>(
            r#"SELECT wallet_address, is_active, created_at, last_auth_at, wallet_metadata FROM wallet_users WHERE lower(wallet_address) = lower($1) LIMIT 1"#,
        )
        .bind(wallet_address)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx wallet find: {e}")))?;
        Ok(row)
    }

    pub async fn list_wallets(
        &self,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxWalletBasicInfo>> {
        let rows = sqlx::query_as::<_, SqlxWalletBasicInfo>(
            r#"SELECT wallet_address, is_active, created_at, last_auth_at, wallet_metadata FROM wallet_users ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx wallet list: {e}")))?;
        Ok(rows)
    }

    pub async fn count_wallets(&self) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM wallet_users"#)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("sqlx wallet count: {e}")))?;
        Ok(count)
    }
}
