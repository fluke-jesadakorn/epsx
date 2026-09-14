//! SQLx Credit Repository — BIG-BANG side-by-side with Diesel.
//!
//! Mirrors `credit_repository_adapter.rs` (Diesel 572 LOC) using `sqlx::PgPool`.
//! Co-exists until last Diesel query removed.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxWalletCreditRow {
    pub wallet_address: String,
    pub balance: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxCreditRepository {
    pool: Arc<PgPool>,
}

impl SqlxCreditRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn get_balance(
        &self,
        wallet_address: &str,
    ) -> AppResult<Option<SqlxWalletCreditRow>> {
        let row = sqlx::query_as::<_, SqlxWalletCreditRow>(
            r#"SELECT wallet_address, balance, created_at, updated_at FROM wallet_credits WHERE wallet_address = $1 LIMIT 1"#,
        )
        .bind(wallet_address)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx credit get_balance: {e}")))?;
        Ok(row)
    }

    pub async fn list_credits(
        &self,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxWalletCreditRow>> {
        let rows = sqlx::query_as::<_, SqlxWalletCreditRow>(
            r#"SELECT wallet_address, balance, created_at, updated_at FROM wallet_credits ORDER BY updated_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx credit list: {e}")))?;
        Ok(rows)
    }

    pub async fn upsert_balance(
        &self,
        wallet_address: &str,
        balance: &str,
    ) -> AppResult<SqlxWalletCreditRow> {
        let row = sqlx::query_as::<_, SqlxWalletCreditRow>(
            r#"
            INSERT INTO wallet_credits (wallet_address, balance, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (wallet_address) DO UPDATE SET balance = EXCLUDED.balance, updated_at = NOW()
            RETURNING wallet_address, balance, created_at, updated_at
            "#,
        )
        .bind(wallet_address)
        .bind(balance)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx credit upsert: {e}")))?;
        Ok(row)
    }
}
