use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use serde::{Serialize};

use crate::infrastructure::adapter_repositories::DbPool;
use crate::schema::{api_key_usage_logs, api_keys};

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

        // 2. Get 24h stats from logs
        // This is heavier, so we only look back 24h
        let one_day_ago = Utc::now() - Duration::days(1);
        
        let requests_24h: i64 = api_key_usage_logs::table
            .inner_join(api_keys::table)
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(api_keys.wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .filter(api_key_usage_logs::request_at.ge(one_day_ago))
            .count()
            .get_result(&mut conn)
            .await?;

        // 3. Calculate success rate (last 1000 requests for approximation)
        // If we scan everything it might be too slow
        let success_rate_query = api_key_usage_logs::table
            .inner_join(api_keys::table)
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(api_keys.wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .order(api_key_usage_logs::request_at.desc())
            .limit(1000)
            .select(api_key_usage_logs::response_status);

        let statuses: Vec<Option<i32>> = success_rate_query.load(&mut conn).await?;
        
        let (success_count, error_count_24h) = if statuses.is_empty() {
            (0, 0)
        } else {
             let errors = statuses.iter().filter(|s| s.map_or(false, |code| code >= 400)).count();
             (statuses.len() - errors, errors)
        };
        
        let average_success_rate = if !statuses.is_empty() {
            (success_count as f64 / statuses.len() as f64) * 100.0
        } else {
            100.0 // Default to 100% if no data
        };

        let error_rate_24h = if !statuses.is_empty() {
             (error_count_24h as f64 / statuses.len() as f64) * 100.0
        } else {
             0.0
        };

        Ok(UsageStats {
            total_requests,
            average_success_rate,
            requests_24h,
            error_rate_24h,
        })
    }

    /// Get usage history (time series)
    pub async fn get_usage_history(
        &self, 
        wallet_address: &str, 
        days: i32
    ) -> Result<Vec<UsageHistoryPoint>, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let wallet_lower = wallet_address.to_lowercase();
        // Use raw SQL for time bucketing as Diesel DSL for date_trunc can be verbose
        // Bucket by day if range > 2 days, else by hour
        let (interval, time_range) = if days > 2 {
            ("day", format!("{} days", days))
        } else {
            ("hour", format!("{} days", days))
        };

        let query = format!("
            SELECT 
                date_trunc('{}', l.request_at) as bucket,
                COUNT(*) as count
            FROM api_key_usage_logs l
            JOIN api_keys k ON l.api_key_id = k.id
            WHERE LOWER(k.wallet_address) = '{}'
            AND l.request_at > NOW() - INTERVAL '{}'
            GROUP BY 1
            ORDER BY 1 ASC
        ", interval, wallet_lower.replace('\'', "''"), time_range);

        let points = diesel::sql_query(query)
            .load::<UsageHistoryPoint>(&mut conn)
            .await?;

        Ok(points)
    }

    /// Get top endpoints
    pub async fn get_top_endpoints(
        &self, 
        wallet_address: &str,
        days: i32
    ) -> Result<Vec<TopEndpoint>, diesel::result::Error> {
         let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;
        
        let wallet_lower = wallet_address.to_lowercase();
        
        let query = format!("
            SELECT 
                l.endpoint,
                l.method,
                COUNT(*) as count
            FROM api_key_usage_logs l
            JOIN api_keys k ON l.api_key_id = k.id
            WHERE LOWER(k.wallet_address) = '{}'
            AND l.request_at > NOW() - INTERVAL '{} days'
            GROUP BY 1, 2
            ORDER BY 3 DESC
            LIMIT 5
        ", wallet_lower.replace('\'', "''"), days);

        let endpoints = diesel::sql_query(query)
            .load::<TopEndpoint>(&mut conn)
            .await?;

        Ok(endpoints)
    }
}
