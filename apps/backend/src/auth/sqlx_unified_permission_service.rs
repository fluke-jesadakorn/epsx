//! SQLx Unified Permission Service — side-by-side with Diesel.
//!
//! Mirrors `unified_permission_service.rs:919` (Diesel) using `sqlx::PgPool`.
//! This file is the canonical pattern for the remaining 117 diesel files:
//! keep Diesel version, add SQLx version, migrate call sites one handler at a time,
//! then delete Diesel version in final commit.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxPermissionRow {
    pub wallet_address: String,
    pub permission_string: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxUnifiedPermissionService {
    pool: Arc<PgPool>,
}

impl SqlxUnifiedPermissionService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn has_permission(&self, wallet_address: &str, permission: &str) -> AppResult<bool> {
        let (exists,): (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM wallet_direct_permissions
                WHERE lower(wallet_address) = lower($1) AND permission_string = $2 AND is_active = true
                UNION ALL
                SELECT 1 FROM wallet_plan_assignments wpa
                JOIN plan_permissions pp ON pp.plan_id = wpa.plan_id
                JOIN permissions p ON p.id = pp.permission_id
                WHERE lower(wpa.wallet_address) = lower($1) AND p.permission_string = $2 AND wpa.is_active = true
                LIMIT 1
            )
            "#,
        )
        .bind(wallet_address)
        .bind(permission)
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx has_permission: {e}")))?;
        Ok(exists)
    }

    pub async fn list_permissions(&self, wallet_address: &str) -> AppResult<Vec<String>> {
        let rows = sqlx::query_as::<_, SqlxPermissionRow>(
            r#"
            SELECT wallet_address, permission_string, is_active, created_at FROM wallet_direct_permissions
            WHERE lower(wallet_address) = lower($1) AND is_active = true
            UNION
            SELECT wpa.wallet_address, p.permission_string, true as is_active, wpa.created_at
            FROM wallet_plan_assignments wpa
            JOIN plan_permissions pp ON pp.plan_id = wpa.plan_id
            JOIN permissions p ON p.id = pp.permission_id
            WHERE lower(wpa.wallet_address) = lower($1) AND wpa.is_active = true
            "#,
        )
        .bind(wallet_address)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx list_permissions: {e}")))?;
        Ok(rows.into_iter().map(|r| r.permission_string).collect())
    }
}
