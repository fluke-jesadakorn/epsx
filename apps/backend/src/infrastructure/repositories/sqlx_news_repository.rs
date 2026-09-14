//! SQLx News Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxNewsRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxNewsRepository {
    pool: Arc<PgPool>,
}

impl SqlxNewsRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_slug(&self, slug: &str) -> AppResult<Option<SqlxNewsRow>> {
        let row = sqlx::query_as::<_, SqlxNewsRow>(
            r#"SELECT id, slug, title, content, is_published, created_at, updated_at FROM news_articles WHERE slug = $1 LIMIT 1"#,
        )
        .bind(slug)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx news find: {e}")))?;
        Ok(row)
    }

    pub async fn list_published(&self, limit: i64, offset: i64) -> AppResult<Vec<SqlxNewsRow>> {
        let rows = sqlx::query_as::<_, SqlxNewsRow>(
            r#"SELECT id, slug, title, content, is_published, created_at, updated_at FROM news_articles WHERE is_published = true ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx news list: {e}")))?;
        Ok(rows)
    }
}
