//! SQLx API Key Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxApiKeyRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxApiKeyRepository {
    pool: Arc<PgPool>,
}

impl SqlxApiKeyRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_prefix(&self, prefix: &str) -> AppResult<Option<SqlxApiKeyRow>> {
        let row = sqlx::query_as::<_, SqlxApiKeyRow>(
            r#"SELECT id, wallet_address, key_prefix, is_active, created_at FROM api_keys WHERE key_prefix = $1 LIMIT 1"#,
        )
        .bind(prefix)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx apikey find: {e}")))?;
        Ok(row)
    }

    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxApiKeyRow>> {
        let rows = sqlx::query_as::<_, SqlxApiKeyRow>(
            r#"SELECT id, wallet_address, key_prefix, is_active, created_at FROM api_keys WHERE wallet_address = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx apikey list: {e}")))?;
        Ok(rows)
    }
}
