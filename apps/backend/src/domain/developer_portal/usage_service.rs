use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use serde::{Serialize};

use crate::infrastructure::adapter_repositories::DbPool;
use crate::schemas::primary::api_keys;

#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub average_success_rate: f64,
    pub requests_24h: i64,
    pub error_rate_24h: f64,
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct UsageHistoryPoint {
    #[diesel(sql_type = Timestamptz)]
    pub bucket: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct TopEndpoint {
    #[diesel(sql_type = Text)]
    pub endpoint: String,
    #[diesel(sql_type = Text)]
    pub method: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

pub struct UsageService {
    pool: DbPool,
}

impl UsageService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get aggregated usage stats for a wallet address
    pub async fn get_wallet_stats(&self, wallet_address: &str) -> Result<UsageStats, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        // 1. Get total requests from api_keys table (more efficient than counting logs)
        let wallet_lower = wallet_address.to_lowercase();
        let total_requests: i64 = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .select(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>>("SUM(total_requests)::BIGINT"))
            .first::<Option<i64>>(&mut conn)
            .await?
            .unwrap_or(0);

        // TODO: Refactor for multi-database support.
        // api_key_usage_logs is now in epsx_analytics_dev, while api_keys is in epsx_dev.
        // We cannot join them or query logs with the main pool.
        // For now, returning 0 for log-based stats.

        Ok(UsageStats {
            total_requests,
            average_success_rate: 100.0,
            requests_24h: 0,
            error_rate_24h: 0.0,
        })
    }

    /// Get usage history (time series)
    pub async fn get_usage_history(
        &self, 
        _wallet_address: &str, 
        _days: i32
    ) -> Result<Vec<UsageHistoryPoint>, diesel::result::Error> {
        // TODO: Refactor for multi-database support.
        Ok(Vec::new())
    }

    /// Get top endpoints
    pub async fn get_top_endpoints(
        &self, 
        _wallet_address: &str,
        _days: i32
    ) -> Result<Vec<TopEndpoint>, diesel::result::Error> {
        // TODO: Refactor for multi-database support.
        Ok(Vec::new())
    }
}
