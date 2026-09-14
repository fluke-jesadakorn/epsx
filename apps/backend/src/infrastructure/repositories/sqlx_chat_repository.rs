//! SQLx Chat Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxChatRow {
    pub id: Uuid,
    pub wallet_address: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxChatRepository {
    pool: Arc<PgPool>,
}

impl SqlxChatRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn list_by_wallet(
        &self,
        wallet_address: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<SqlxChatRow>> {
        let rows = sqlx::query_as::<_, SqlxChatRow>(
            r#"SELECT id, wallet_address, message, created_at FROM chat_messages WHERE wallet_address = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(wallet_address)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx chat list: {e}")))?;
        Ok(rows)
    }

    pub async fn save(&self, wallet_address: &str, message: &str) -> AppResult<SqlxChatRow> {
        let row = sqlx::query_as::<_, SqlxChatRow>(
            r#"INSERT INTO chat_messages (id, wallet_address, message, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id, wallet_address, message, created_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(wallet_address)
        .bind(message)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx chat save: {e}")))?;
        Ok(row)
    }
}
