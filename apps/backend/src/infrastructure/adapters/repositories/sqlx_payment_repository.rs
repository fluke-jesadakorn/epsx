//! SQLx Payment Repository — BIG-BANG Phase A example (side-by-side with Diesel).
//!
//! Mirrors `payment_repository_adapter.rs` (Diesel, 665 LOC) but uses `sqlx::PgPool`.
//! This file is the canonical pattern for the big-bang migration: one method
//! at a time, side-by-side, until the last Diesel query is removed.
//!
//! BIG-BANG: co-exists with `PaymentRepositoryAdapter` (Diesel). The `DbPool`
//! alias (`&'static TlsPool`) is deprecated; new code uses `Arc<PgPool>`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

/// Row for `payments` table (sqlx version) — mirrors `PaymentDb` in `models/payment.rs`
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxPaymentRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub transaction_hash: Option<String>,
    pub chain_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLx implementation — replacement for `PaymentRepositoryAdapter`
#[derive(Clone)]
pub struct SqlxPaymentRepository {
    pool: Arc<PgPool>,
}

impl SqlxPaymentRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Find payment by id (mirrors Diesel `payments::table.find(id).first(&mut conn)`)
    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SqlxPaymentRow>> {
        let row = sqlx::query_as::<_, SqlxPaymentRow>(
            r#"
            SELECT id, wallet_address, amount, currency, status, transaction_hash, chain_id, created_at, updated_at
            FROM payments
            WHERE id = $1
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx payment find_by_id: {e}")))?;
        Ok(row)
    }

    /// List payments by wallet (case-insensitive, paginated)
    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxPaymentRow>> {
        let rows = sqlx::query_as::<_, SqlxPaymentRow>(
            r#"
            SELECT id, wallet_address, amount, currency, status, transaction_hash, chain_id, created_at, updated_at
            FROM payments
            WHERE lower(wallet_address) = lower($1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx payment list_by_wallet: {e}")))?;
        Ok(rows)
    }

    /// Insert new payment (mirrors Diesel `insert_into(payments).values(&new).get_result()`)
    pub async fn save(&self, row: SqlxPaymentRow) -> AppResult<SqlxPaymentRow> {
        let inserted = sqlx::query_as::<_, SqlxPaymentRow>(
            r#"
            INSERT INTO payments (id, wallet_address, amount, currency, status, transaction_hash, chain_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, wallet_address, amount, currency, status, transaction_hash, chain_id, created_at, updated_at
            "#,
        )
        .bind(row.id)
        .bind(&row.wallet_address)
        .bind(&row.amount)
        .bind(&row.currency)
        .bind(&row.status)
        .bind(&row.transaction_hash)
        .bind(&row.chain_id)
        .bind(row.created_at)
        .bind(row.updated_at)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx payment save: {e}")))?;
        Ok(inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlx_payment_row_shape() {
        let _row = SqlxPaymentRow {
            id: Uuid::new_v4(),
            wallet_address: "0xabc".to_string(),
            amount: "100".to_string(),
            currency: "USDT".to_string(),
            status: "created".to_string(),
            transaction_hash: None,
            chain_id: Some("56".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(_row.currency, "USDT");
    }
}
