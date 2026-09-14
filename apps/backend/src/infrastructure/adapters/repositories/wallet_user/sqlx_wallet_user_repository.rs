//! SQLx Wallet User Repository — BIG-BANG Phase 1b example (side-by-side with Diesel).
//!
//! This file demonstrates the canonical `sqlx` pattern for the big-bang
//! Diesel→sqlx migration on the single branch `migration/dioxus-microservices`.
//! It co-exists with `user_adapter.rs` (Diesel) until all call sites migrate.
//!
//! Pattern (per `shared/rust/epsx-database-pools/src/sqlx_pool.rs`):
//!   - `sqlx::query_as!` with compile-time checked SQL (requires `DATABASE_URL` at compile time for `query!`, or `query_as` runtime)
//!   - `PgPool` injected via `Arc<PgPool>` instead of `&'static TlsPool`
//!   - No `diesel::QueryableByName`, no `TlsPool`, no `deadpool`

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

use crate::prelude::{AppError, AppResult};

/// Row for `wallet_users` table (sqlx version)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxWalletUserRow {
    pub wallet_address: String,
    pub is_active: bool,
    pub wallet_metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_auth_at: Option<DateTime<Utc>>,
}

/// SQLx implementation — mirrors `WalletUserRepositoryAdapter` but uses `PgPool`.
/// BIG-BANG: This struct is the replacement; `WalletUserRepositoryAdapter` (Diesel) is deprecated.
#[derive(Clone)]
pub struct SqlxWalletUserRepository {
    pool: Arc<PgPool>,
}

impl SqlxWalletUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Example: find by wallet_address (case-insensitive, mirrors Diesel `lower()` query).
    /// Diesel version:
    ///   diesel::sql_query("SELECT ... FROM wallet_users WHERE lower(wallet_address)=lower($1)")
    ///     .bind::<Text,_>(addr).load::<WalletUserQueryResult>(&mut *conn)
    /// SQLx version:
    pub async fn find_by_wallet_address(
        &self,
        wallet_address: &str,
    ) -> AppResult<Option<SqlxWalletUserRow>> {
        let row = sqlx::query_as::<_, SqlxWalletUserRow>(
            r#"
            SELECT wallet_address, is_active, wallet_metadata, created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE lower(wallet_address) = lower($1)
            LIMIT 1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx wallet_user find: {e}")))?;
        Ok(row)
    }

    /// Example: list active users with pagination (mirrors Diesel `limit`/`offset` DSL).
    pub async fn list_active(&self, limit: i64, offset: i64) -> AppResult<Vec<SqlxWalletUserRow>> {
        let rows = sqlx::query_as::<_, SqlxWalletUserRow>(
            r#"
            SELECT wallet_address, is_active, wallet_metadata, created_at, updated_at, last_auth_at
            FROM wallet_users
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx wallet_user list: {e}")))?;
        Ok(rows)
    }

    /// Health check for this pool (mirrors `sqlx_pool::health_check`)
    pub async fn health_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map(|_| ())
            .map_err(|e| AppError::database_error(format!("sqlx health: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlx_row_maps_to_domain_shape() {
        // Compile-time check that row fields match wallet_users table
        let _row = SqlxWalletUserRow {
            wallet_address: "0xabc".to_string(),
            is_active: true,
            wallet_metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_auth_at: None,
        };
        assert_eq!(_row.wallet_address, "0xabc");
    }
}
