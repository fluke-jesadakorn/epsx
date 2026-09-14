//! SQLx Ranking Entitlement Repository — side-by-side with Diesel.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::{AppError, AppResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SqlxRankingEntitlementRow {
    pub wallet_address: String,
    pub plan_id: Uuid,
    pub ranking_offset: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SqlxRankingEntitlementRepository {
    pool: Arc<PgPool>,
}

impl SqlxRankingEntitlementRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn find_by_wallet(
        &self,
        wallet_address: &str,
    ) -> AppResult<Vec<SqlxRankingEntitlementRow>> {
        let rows = sqlx::query_as::<_, SqlxRankingEntitlementRow>(
            r#"SELECT wallet_address, plan_id, ranking_offset, created_at FROM ranking_entitlements WHERE wallet_address = $1 ORDER BY created_at DESC"#,
        )
        .bind(wallet_address)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("sqlx ranking find: {e}")))?;
        Ok(rows)
    }
}
