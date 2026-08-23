//! SQLx Stock Analysis Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxStockAnalysisRow {
    pub id: Uuid,
    pub symbol: String,
    pub eps: Option<String>,
    pub growth: Option<String>,
    pub sector: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxStockAnalysisRepository {
    pool: Arc<PgPool>,
}

impl SqlxStockAnalysisRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_symbol(&self, symbol: &str) -> AppResult<Option<SqlxStockAnalysisRow>> {
        let row = sqlx::query_as::<_, SqlxStockAnalysisRow>(
            r#"SELECT id, symbol, eps, growth, sector, created_at, updated_at FROM stock_analyses WHERE symbol = $1 LIMIT 1"#,
        )
        .bind(symbol)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx stock find: {e}")))?;
        Ok(row)
    }

    pub async fn list_by_sector(
        &self,
        sector: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxStockAnalysisRow>> {
        let rows = sqlx::query_as::<_, SqlxStockAnalysisRow>(
            r#"SELECT id, symbol, eps, growth, sector, created_at, updated_at FROM stock_analyses WHERE sector = $1 ORDER BY symbol ASC LIMIT $2 OFFSET $3"#,
        )
        .bind(sector)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx stock list: {e}")))?;
        Ok(rows)
    }
}
