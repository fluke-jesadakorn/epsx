//! SQLx Audit Log Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxAuditLogRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub action: String,
    pub resource_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxAuditLogRepository {
    pool: Arc<PgPool>,
}

impl SqlxAuditLogRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        wallet_address: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxAuditLogRow>> {
        let rows = if let Some(addr) = wallet_address {
            sqlx::query_as::<_, SqlxAuditLogRow>(
                r#"SELECT id, wallet_address, action, resource_type, created_at FROM audit_logs WHERE wallet_address = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            )
            .bind(addr)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("sqlx audit list: {e}")))?
        } else {
            sqlx::query_as::<_, SqlxAuditLogRow>(
                r#"SELECT id, wallet_address, action, resource_type, created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("sqlx audit list: {e}")))?
        };
        Ok(rows)
    }

    pub async fn count(&self, wallet_address: Option<&str>) -> AppResult<i64> {
        let (count,): (i64,) = if let Some(addr) = wallet_address {
            sqlx::query_as(r#"SELECT COUNT(*) FROM audit_logs WHERE wallet_address = $1"#)
                .bind(addr)
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(|e| AppError::database_error(format!("sqlx audit count: {e}")))?
        } else {
            sqlx::query_as(r#"SELECT COUNT(*) FROM audit_logs"#)
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(|e| AppError::database_error(format!("sqlx audit count: {e}")))?
        };
        Ok(count)
    }
}
