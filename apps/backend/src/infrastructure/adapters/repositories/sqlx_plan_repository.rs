//! SQLx Plan Repository — BIG-BANG side-by-side with Diesel.
//!
//! Mirrors `permission_plan_repository_adapter.rs` (Diesel 583 LOC) using `sqlx::PgPool`.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxPlanRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub plan_type: String,
    pub tier_level: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct SqlxPlanRepository {
    pool: Arc<PgPool>,
}

impl SqlxPlanRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SqlxPlanRow>> {
        let row = sqlx::query_as::<_, SqlxPlanRow>(
            r#"SELECT id, name, slug, is_active, plan_type, tier_level, created_at, updated_at, metadata FROM plans WHERE id = $1 LIMIT 1"#,
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx plan find_by_id: {e}")))?;
        Ok(row)
    }

    pub async fn find_by_slug(&self, slug: &str) -> AppResult<Option<SqlxPlanRow>> {
        let row = sqlx::query_as::<_, SqlxPlanRow>(
            r#"SELECT id, name, slug, is_active, plan_type, tier_level, created_at, updated_at, metadata FROM plans WHERE slug = $1 LIMIT 1"#,
        )
        .bind(slug)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx plan find_by_slug: {e}")))?;
        Ok(row)
    }

    pub async fn list_active(&self, limit: i64, offset: i64) -> AppResult<Vec<SqlxPlanRow>> {
        let rows = sqlx::query_as::<_, SqlxPlanRow>(
            r#"SELECT id, name, slug, is_active, plan_type, tier_level, created_at, updated_at, metadata FROM plans WHERE is_active = true ORDER BY tier_level DESC, name ASC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx plan list: {e}")))?;
        Ok(rows)
    }

    pub async fn count_active(&self) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM plans WHERE is_active = true"#,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx plan count: {e}")))?;
        Ok(count)
    }
}
