//! SQLx Notification Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxWalletNotificationRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub title: String,
    pub body: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxNotificationRepository {
    pool: Arc<PgPool>,
}

impl SqlxNotificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxWalletNotificationRow>> {
        let rows = sqlx::query_as::<_, SqlxWalletNotificationRow>(
            r#"SELECT id, wallet_address, title, body, is_read, created_at FROM wallet_notifications WHERE wallet_address = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx notification list: {e}")))?;
        Ok(rows)
    }

    pub async fn mark_read(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(r#"UPDATE wallet_notifications SET is_read = true WHERE id = $1"#)
            .bind(id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("sqlx notification mark_read: {e}")))?;
        Ok(())
    }

    pub async fn count_unread(&self, wallet_address: &str) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM wallet_notifications WHERE wallet_address = $1 AND is_read = false"#,
        )
        .bind(wallet_address)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx notification count: {e}")))?;
        Ok(count)
    }
}
