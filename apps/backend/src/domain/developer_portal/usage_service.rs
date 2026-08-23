use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamptz};
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::adapter_repositories::DbPool;
use crate::schemas::infra_logs::api_key_usage_logs;
use crate::schemas::primary::{api_keys, api_modules};

/// API usage statistics for a wallet
#[derive(Debug, Serialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub average_success_rate: f64,
    pub requests_24h: i64,
    pub error_rate_24h: f64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct UsageReport {
    pub days: i32,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub error_requests: i64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub daily: Vec<DailyUsage>,
    pub top_endpoints: Vec<EndpointReport>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub total_requests: i64,
    pub error_requests: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct EndpointReport {
    pub endpoint: String,
    pub method: String,
    pub request_count: i64,
    pub error_count: i64,
    pub average_response_time_ms: f64,
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
    pub unique_api_keys: i64,
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

fn pool_error(error: impl std::fmt::Display) -> diesel::result::Error {
    diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new(error.to_string()),
    )
}

fn empty_report(days: i32, start_date: NaiveDate) -> UsageReport {
    UsageReport {
        days,
        total_requests: 0,
        successful_requests: 0,
        error_requests: 0,
        success_rate: 0.0,
        error_rate: 0.0,
        average_response_time_ms: 0.0,
        daily: (0..days)
            .map(|offset| DailyUsage {
                date: start_date + Duration::days(i64::from(offset)),
                total_requests: 0,
                error_requests: 0,
            })
            .collect(),
        top_endpoints: Vec::new(),
    }
}

impl UsageService {
    /// Create a new usage service with dual database pools
    pub fn new(core_pool: DbPool, analytics_pool: DbPool) -> Self {
        Self {
            core_pool,
            analytics_pool,
        }
    }

    /// Create a usage service with only core pool (legacy compatibility, limited functionality)
    /// Note: Analytics queries will use core pool and likely fail if tables don't exist
    pub fn new_core_only(core_pool: DbPool) -> Self {
        Self {
            core_pool,
            analytics_pool: core_pool, // Same pool, will fail on analytics-specific tables
        }
    }

    pub async fn get_report(
        &self,
        wallet_address: &str,
        days: i32,
    ) -> Result<UsageReport, diesel::result::Error> {
        if !matches!(days, 7 | 30 | 90) {
            return Err(diesel::result::Error::NotFound);
        }
        let mut core_conn = self.core_pool.acquire().await.map_err(pool_error)?;
        let api_key_ids = api_keys::table
            .filter(api_keys::wallet_address.ilike(wallet_address))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;
        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(i64::from(days - 1));
        if api_key_ids.is_empty() {
            return Ok(empty_report(days, start_date));
        }

        #[derive(QueryableByName)]
        struct TotalsRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_requests: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            successful_requests: i64,
            #[diesel(sql_type = diesel::sql_types::Double)]
            average_response_time_ms: f64,
        }
        #[derive(QueryableByName)]
        struct DailyRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_requests: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            error_requests: i64,
        }
        #[derive(QueryableByName)]
        struct EndpointRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            endpoint: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            method: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            request_count: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            error_count: i64,
            #[diesel(sql_type = diesel::sql_types::Double)]
            average_response_time_ms: f64,
        }

        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(pool_error)?;
        let totals = diesel::sql_query(
            r#"
            SELECT COUNT(*)::bigint AS total_requests,
                   COUNT(*) FILTER (WHERE response_status < 400)::bigint AS successful_requests,
                   COALESCE(AVG(response_time_ms), 0)::float8 AS average_response_time_ms
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            "#,
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&api_key_ids)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .get_result::<TotalsRow>(&mut analytics_conn)
        .await?;
        let daily_rows = diesel::sql_query(
            r#"
            SELECT request_at::date AS date,
                   COUNT(*)::bigint AS total_requests,
                   COUNT(*) FILTER (WHERE response_status IS NULL OR response_status >= 400)::bigint AS error_requests
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            GROUP BY request_at::date
            ORDER BY date ASC
            "#,
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&api_key_ids)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .load::<DailyRow>(&mut analytics_conn)
        .await?;
        let endpoint_rows = diesel::sql_query(
            r#"
            SELECT endpoint::text, method::text,
                   COUNT(*)::bigint AS request_count,
                   COUNT(*) FILTER (WHERE response_status IS NULL OR response_status >= 400)::bigint AS error_count,
                   COALESCE(AVG(response_time_ms), 0)::float8 AS average_response_time_ms
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            GROUP BY endpoint, method
            ORDER BY request_count DESC, method ASC, endpoint ASC
            LIMIT 10
            "#,
        )
        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&api_key_ids)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .load::<EndpointRow>(&mut analytics_conn)
        .await?;

        let daily_by_date = daily_rows
            .into_iter()
            .map(|row| (row.date, row))
            .collect::<std::collections::HashMap<_, _>>();
        let daily = (0..days)
            .map(|offset| {
                let date = start_date + Duration::days(i64::from(offset));
                daily_by_date.get(&date).map_or(
                    DailyUsage {
                        date,
                        total_requests: 0,
                        error_requests: 0,
                    },
                    |row| DailyUsage {
                        date,
                        total_requests: row.total_requests,
                        error_requests: row.error_requests,
                    },
                )
            })
            .collect();
        let error_requests = totals.total_requests - totals.successful_requests;
        let (success_rate, error_rate) = if totals.total_requests == 0 {
            (0.0, 0.0)
        } else {
            let success = totals.successful_requests as f64 * 100.0 / totals.total_requests as f64;
            (success, 100.0 - success)
        };

        Ok(UsageReport {
            days,
            total_requests: totals.total_requests,
            successful_requests: totals.successful_requests,
            error_requests,
            success_rate,
            error_rate,
            average_response_time_ms: totals.average_response_time_ms,
            daily,
            top_endpoints: endpoint_rows
                .into_iter()
                .map(|row| EndpointReport {
                    endpoint: row.endpoint,
                    method: row.method,
                    request_count: row.request_count,
                    error_count: row.error_count,
                    average_response_time_ms: row.average_response_time_ms,
                })
                .collect(),
        })
    }

    /// Get aggregated usage stats for a wallet address
    pub async fn get_wallet_stats(
        &self,
        wallet_address: &str,
    ) -> Result<UsageStats, diesel::result::Error> {
        let mut core_conn = self.core_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        // Get API key IDs for this wallet from core database
        let wallet_lower = wallet_address.to_lowercase();
        let total_requests: i64 = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'",
                wallet_lower.replace('\'', "''")
            )))
            .select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
                "COALESCE(SUM(total_requests), 0)::BIGINT",
            ))
            .first::<i64>(&mut core_conn)
            .await?;

        // Get API key IDs for analytics queries
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'",
                wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            // A wallet without keys has no measured success rate. Returning
            // 100% here would fabricate an operational metric from absence of
            // data; the admin read surface maps this dependency result to an
            // explicit unavailable state.
            return Err(diesel::result::Error::NotFound);
        }

        // Query analytics database for 24h stats
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
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
            .await?;

        // Count error requests in last 24 hours (status >= 400)
        let error_count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::api_key_id.eq_any(&api_key_ids))
            .filter(api_key_usage_logs::request_at.ge(&twenty_four_hours_ago))
            .filter(api_key_usage_logs::response_status.ge(Some(400)))
            .count()
            .get_result(&mut analytics_conn)
            .await?;

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
        days: i32,
    ) -> Result<Vec<UsageHistoryPoint>, diesel::result::Error> {
        // Get API key IDs from core database
        let mut core_conn = self.core_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let wallet_lower = wallet_address.to_lowercase();
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'",
                wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query analytics database for time series
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let start_date = Utc::now() - Duration::days(days as i64);

        // Generate time buckets (daily) with request counts
        // Using raw SQL for proper time bucketing
        let api_key_ids_str: String = api_key_ids
            .iter()
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
        .await?;

        Ok(history)
    }

    /// Get top endpoints for a wallet
    pub async fn get_top_endpoints(
        &self,
        wallet_address: &str,
        days: i32,
    ) -> Result<Vec<TopEndpoint>, diesel::result::Error> {
        // Get API key IDs from core database
        let mut core_conn = self.core_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let wallet_lower = wallet_address.to_lowercase();
        let api_key_ids: Vec<Uuid> = api_keys::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                "LOWER(wallet_address) = '{}'",
                wallet_lower.replace('\'', "''")
            )))
            .select(api_keys::id)
            .load::<Uuid>(&mut core_conn)
            .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Query analytics database for top endpoints
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let start_date = Utc::now() - Duration::days(days as i64);

        let api_key_ids_str: String = api_key_ids
            .iter()
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
        .await?;

        Ok(top_endpoints)
    }

    /// Get today's total request count (for admin stats)
    pub async fn get_requests_today(&self) -> Result<i64, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let today_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("midnight is always a valid time");

        let count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::request_at.ge(&today_start))
            .count()
            .get_result(&mut analytics_conn)
            .await?;

        Ok(count)
    }

    /// Get this month's total request count (for admin stats)
    pub async fn get_requests_this_month(&self) -> Result<i64, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let now = Utc::now();
        let month_start = now
            .date_naive()
            .with_day(1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("the first day of a month at midnight is always valid");

        let count: i64 = api_key_usage_logs::table
            .filter(api_key_usage_logs::request_at.ge(&month_start))
            .count()
            .get_result(&mut analytics_conn)
            .await?;

        Ok(count)
    }

    /// Get top modules by usage (for admin stats)
    pub async fn get_top_modules_by_usage(
        &self,
        limit: i64,
    ) -> Result<Vec<ModuleUsage>, diesel::result::Error> {
        let mut analytics_conn = self.analytics_pool.acquire().await.map_err(|e| {
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
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            unique_api_keys: i64,
        }

        let month_start = Utc::now()
            .date_naive()
            .with_day(1)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc))
            .expect("the first day of a month at midnight is always valid");

        let module_counts: Vec<ModuleCount> = diesel::sql_query(format!(
            r#"
            SELECT 
                module_id,
                COUNT(*)::BIGINT as count,
                COUNT(DISTINCT api_key_id)::BIGINT as unique_api_keys
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
        .await?;

        if module_counts.is_empty() {
            return Ok(Vec::new());
        }

        let module_ids: Vec<Uuid> = module_counts
            .iter()
            .map(|module| module.module_id)
            .collect();
        let mut core_conn = self.core_pool.acquire().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;
        let module_rows: Vec<(Uuid, String)> = api_modules::table
            .filter(api_modules::id.eq_any(&module_ids))
            .select((api_modules::id, api_modules::display_name))
            .load(&mut core_conn)
            .await?;
        let module_names: std::collections::HashMap<Uuid, String> =
            module_rows.into_iter().collect();
        if module_names.len() != module_ids.len() {
            return Err(diesel::result::Error::NotFound);
        }

        let modules: Vec<ModuleUsage> = module_counts
            .into_iter()
            .map(|mc| {
                let module_name = module_names
                    .get(&mc.module_id)
                    .cloned()
                    .ok_or(diesel::result::Error::NotFound)?;
                Ok(ModuleUsage {
                    module_id: mc.module_id,
                    module_name,
                    request_count: mc.count,
                    unique_api_keys: mc.unique_api_keys,
                })
            })
            .collect::<Result<_, diesel::result::Error>>()?;

        Ok(modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_usage_is_explicit_and_has_every_day_bucket() {
        let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        for days in [7, 30, 90] {
            let report = empty_report(days, start);
            assert_eq!(report.days, days);
            assert_eq!(report.daily.len(), days as usize);
            assert_eq!(
                report.daily.as_slice().first().map(|point| point.date),
                Some(start)
            );
            assert_eq!(
                report.daily.as_slice().last().map(|point| point.date),
                Some(start + Duration::days(i64::from(days - 1)))
            );
            assert_eq!(report.total_requests, 0);
            assert!(report.top_endpoints.is_empty());
        }
    }
}
