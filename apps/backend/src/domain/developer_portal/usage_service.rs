use chrono::{DateTime, Datelike, Duration, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::adapter_repositories::DbPool;
use crate::schemas::primary::api_keys;
use crate::schemas::infra_logs::api_key_usage_logs;

/// API usage statistics for a wallet
#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub average_success_rate: f64,
    pub requests_24h: i64,
    pub error_rate_24h: f64,
}

/// Time-bucketed usage data point
#[derive(Debug, Serialize, QueryableByName)]
pub struct UsageHistoryPoint {
    #[diesel(sql_type = Timestamptz)]
    pub bucket: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

/// Top endpoint statistics
#[derive(Debug, Serialize, QueryableByName)]
pub struct TopEndpoint {
    #[diesel(sql_type = Text)]
    pub endpoint: String,
    #[diesel(sql_type = Text)]
    pub method: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}

/// Module usage for stats
#[derive(Debug, Serialize)]
pub struct ModuleUsage {
    pub module_id: Uuid,
    pub module_name: String,
    pub request_count: i64,
}

/// Usage service with multi-database support
/// 
/// This service queries:
/// - `core_pool`: For `api_keys` table (key metadata)
/// - `analytics_pool`: For `api_key_usage_logs` table (usage metrics)
pub struct UsageService {
    core_pool: DbPool,
    analytics_pool: DbPool,
}

impl UsageService {
    /// Create a new usage service with dual database pools
    pub fn new(core_pool: DbPool, analytics_pool: DbPool) -> Self {
        Self { core_pool, analytics_pool }
    }

    /// Create a usage service with only core pool (legacy compatibility, limited functionality)
    /// Note: Analytics queries will use core pool and likely fail if tables don't exist
    pub fn new_core_only(core_pool: DbPool) -> Self {
        Self { 
            core_pool, 
            analytics_pool: core_pool  // Same pool, will fail on analytics-specific tables
        }
    }

    /// Get aggregated usage stats for a wallet address
    pub async fn get_wallet_stats(&self, wallet_address: &str) -> Result<UsageStats, diesel::result::Error> {
        let mut core_conn = self.core_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        // Get API key IDs for this wallet from core database
        let wallet_lower = wallet_address.to_lowercase();
        let total_requests: i64 = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .select(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>>("SUM(total_requests)::BIGINT"))
            .first::<Option<i64>>(&mut core_conn)
            .await?
            .unwrap_or(0);

        // Get API key IDs for analytics queries
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            return Ok(UsageStats {
                total_requests,
                average_success_rate: 100.0,
                requests_24h: 0,
                error_rate_24h: 0.0,
            });
        }

        // Query analytics database for 24h stats
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now();
        let twenty_four_hours_ago = now - Duration::hours(24);

        // Count requests in last 24 hours
        let requests_24h: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::api_key_id.eq_any(&api_key_ids))
            .filter(api_key_usage_logs::request_at.ge(&twenty_four_hours_ago))
            .count()
            .get_result(&mut analytics_conn)
            .await
            .unwrap_or(0);

        // Count error requests in last 24 hours (status >= 400)
        let error_count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::api_key_id.eq_any(&api_key_ids))
            .filter(api_key_usage_logs::request_at.ge(&twenty_four_hours_ago))
            .filter(api_key_usage_logs::response_status.ge(Some(400)))
            .count()
            .get_result(&mut analytics_conn)
            .await
            .unwrap_or(0);

        // Calculate rates
        let error_rate_24h = if requests_24h > 0 {
            (error_count as f64 / requests_24h as f64) * 100.0
        } else {
            0.0
        };

        let average_success_rate = 100.0 - error_rate_24h;

        Ok(UsageStats {
            total_requests,
            average_success_rate,
            requests_24h,
            error_rate_24h,
        })
    }

    /// Get usage history (time series) for a wallet
    pub async fn get_usage_history(
        &self, 
        wallet_address: &str, 
        days: i32
    ) -> Result<Vec<UsageHistoryPoint>, diesel::result::Error> {
        // Get API key IDs from core database
        let mut core_conn = self.core_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let wallet_lower = wallet_address.to_lowercase();
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query analytics database for time series
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let start_date = Utc::now() - Duration::days(days as i64);
        
        // Generate time buckets (daily) with request counts
        // Using raw SQL for proper time bucketing
        let api_key_ids_str: String = api_key_ids.iter()
            .map(|id| format!("'{}'", id))
            .collect::<Vec<_>>()
            .join(",");

        let history: Vec<UsageHistoryPoint> = diesel::sql_query(format!(
            r#"
            SELECT 
                date_trunc('day', request_at) as bucket,
                COUNT(*)::BIGINT as count
            FROM api_key_usage_logs
            WHERE api_key_id IN ({})
              AND request_at >= $1
            GROUP BY date_trunc('day', request_at)
            ORDER BY bucket DESC
            "#,
            api_key_ids_str
        ))
        .bind::<Timestamptz, _>(start_date)
        .load::<UsageHistoryPoint>(&mut analytics_conn)
        .await
        .unwrap_or_default();

        Ok(history)
    }

    /// Get top endpoints for a wallet
    pub async fn get_top_endpoints(
        &self, 
        wallet_address: &str,
        days: i32
    ) -> Result<Vec<TopEndpoint>, diesel::result::Error> {
        // Get API key IDs from core database
        let mut core_conn = self.core_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let wallet_lower = wallet_address.to_lowercase();
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'", wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query analytics database for top endpoints
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let start_date = Utc::now() - Duration::days(days as i64);
        
        let api_key_ids_str: String = api_key_ids.iter()
            .map(|id| format!("'{}'", id))
            .collect::<Vec<_>>()
            .join(",");

        let top_endpoints: Vec<TopEndpoint> = diesel::sql_query(format!(
            r#"
            SELECT 
                endpoint,
                method,
                COUNT(*)::BIGINT as count
            FROM api_key_usage_logs
            WHERE api_key_id IN ({})
              AND request_at >= $1
            GROUP BY endpoint, method
            ORDER BY count DESC
            LIMIT 10
            "#,
            api_key_ids_str
        ))
        .bind::<Timestamptz, _>(start_date)
        .load::<TopEndpoint>(&mut analytics_conn)
        .await
        .unwrap_or_default();

        Ok(top_endpoints)
    }

    /// Get today's total request count (for admin stats)
    pub async fn get_requests_today(&self) -> Result<i64, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .unwrap_or_else(Utc::now);

        let count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::request_at.ge(&today_start))
            .count()
            .get_result(&mut analytics_conn)
            .await
            .unwrap_or(0);

        Ok(count)
    }

    /// Get this month's total request count (for admin stats)
    pub async fn get_requests_this_month(&self) -> Result<i64, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now();
        let month_start = now.date_naive()
            .with_day(1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .unwrap_or_else(Utc::now);

        let count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::request_at.ge(&month_start))
            .count()
            .get_result(&mut analytics_conn)
            .await
            .unwrap_or(0);

        Ok(count)
    }

    /// Get top modules by usage (for admin stats)
    pub async fn get_top_modules_by_usage(&self, limit: i64) -> Result<Vec<ModuleUsage>, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        // Query for module usage counts
        #[derive(QueryableByName)]
        struct ModuleCount {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            module_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let month_start = Utc::now().date_naive()
            .with_day(1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .unwrap_or_else(Utc::now);

        let module_counts: Vec<ModuleCount> = diesel::sql_query(format!(
            r#"
            SELECT 
                module_id,
                COUNT(*)::BIGINT as count
            FROM api_key_usage_logs
            WHERE request_at >= $1
              AND module_id IS NOT NULL
            GROUP BY module_id
            ORDER BY count DESC
            LIMIT {}
            "#,
            limit
        ))
        .bind::<Timestamptz, _>(month_start)
        .load::<ModuleCount>(&mut analytics_conn)
        .await
        .unwrap_or_default();

        // Convert to ModuleUsage (module name lookup would require core DB join)
        let modules: Vec<ModuleUsage> = module_counts.into_iter()
            .map(|mc| ModuleUsage {
                module_id: mc.module_id,
                module_name: format!("module-{}", mc.module_id.to_string().chars().take(8).collect::<String>()),
                request_count: mc.count,
            })
            .collect();

        Ok(modules)
    }
}
