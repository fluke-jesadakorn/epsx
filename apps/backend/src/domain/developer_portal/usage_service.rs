// BIG-BANG: migrated to sqlx (real). Previously leaked DbPool + diesel DSL.
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

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
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UsageHistoryPoint {
    pub bucket: DateTime<Utc>,
    pub count: i64,
}

/// Top endpoint statistics
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TopEndpoint {
    pub endpoint: String,
    pub method: String,
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
pub struct UsageService {
    core_pool: PgPool,
    analytics_pool: PgPool,
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
    pub fn new(core_pool: PgPool, analytics_pool: PgPool) -> Self {
        Self {
            core_pool,
            analytics_pool,
        }
    }

    pub fn new_core_only(core_pool: PgPool) -> Self {
        Self {
            core_pool,
            analytics_pool: core_pool,
        }
    }

    pub async fn get_report(
        &self,
        wallet_address: &str,
        days: i32,
    ) -> Result<UsageReport, sqlx::Error> {
        if !matches!(days, 7 | 30 | 90) {
            return Err(sqlx::Error::RowNotFound);
        }

        let api_key_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM api_keys WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address)
        .fetch_all(&self.core_pool)
        .await?;

        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(i64::from(days - 1));

        if api_key_ids.is_empty() {
            return Ok(empty_report(days, start_date));
        }

        #[derive(sqlx::FromRow)]
        struct TotalsRow {
            total_requests: i64,
            successful_requests: i64,
            average_response_time_ms: f64,
        }
        #[derive(sqlx::FromRow)]
        struct DailyRow {
            date: NaiveDate,
            total_requests: i64,
            error_requests: i64,
        }
        #[derive(sqlx::FromRow)]
        struct EndpointRow {
            endpoint: String,
            method: String,
            request_count: i64,
            error_count: i64,
            average_response_time_ms: f64,
        }

        let totals: TotalsRow = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint AS total_requests,
                   COUNT(*) FILTER (WHERE response_status < 400)::bigint AS successful_requests,
                   COALESCE(AVG(response_time_ms), 0)::float8 AS average_response_time_ms
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            "#,
        )
        .bind(&api_key_ids)
        .bind(start_date)
        .fetch_one(&self.analytics_pool)
        .await?;

        let daily_rows: Vec<DailyRow> = sqlx::query_as(
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
        .bind(&api_key_ids)
        .bind(start_date)
        .fetch_all(&self.analytics_pool)
        .await?;

        let endpoint_rows: Vec<EndpointRow> = sqlx::query_as(
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
        .bind(&api_key_ids)
        .bind(start_date)
        .fetch_all(&self.analytics_pool)
        .await?;

        let total_requests = totals.total_requests;
        let successful_requests = totals.successful_requests;
        let error_requests = (total_requests - successful_requests).max(0);
        let success_rate = if total_requests > 0 {
            successful_requests as f64 / total_requests as f64
        } else {
            0.0
        };
        let error_rate = if total_requests > 0 {
            error_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        let daily: Vec<DailyUsage> = daily_rows
            .into_iter()
            .map(|row| DailyUsage {
                date: row.date,
                total_requests: row.total_requests,
                error_requests: row.error_requests,
            })
            .collect();

        let top_endpoints: Vec<EndpointReport> = endpoint_rows
            .into_iter()
            .map(|row| EndpointReport {
                endpoint: row.endpoint,
                method: row.method,
                request_count: row.request_count,
                error_count: row.error_count,
                average_response_time_ms: row.average_response_time_ms,
            })
            .collect();

        Ok(UsageReport {
            days,
            total_requests,
            successful_requests,
            error_requests,
            success_rate,
            error_rate,
            average_response_time_ms: totals.average_response_time_ms,
            daily,
            top_endpoints,
        })
    }

    pub async fn get_daily_history(
        &self,
        wallet_address: &str,
        days: i32,
    ) -> Result<Vec<UsageHistoryPoint>, sqlx::Error> {
        if !matches!(days, 7 | 30 | 90) {
            return Ok(Vec::new());
        }

        let api_key_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM api_keys WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address)
        .fetch_all(&self.core_pool)
        .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(i64::from(days - 1));

        let points: Vec<UsageHistoryPoint> = sqlx::query_as(
            r#"
            SELECT date_trunc('day', request_at) AS bucket,
                   COUNT(*)::bigint AS count
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            GROUP BY bucket
            ORDER BY bucket ASC
            "#,
        )
        .bind(&api_key_ids)
        .bind(start_date)
        .fetch_all(&self.analytics_pool)
        .await?;

        Ok(points)
    }

    pub async fn get_top_endpoints(
        &self,
        wallet_address: &str,
        days: i32,
        limit: i64,
    ) -> Result<Vec<TopEndpoint>, sqlx::Error> {
        if !matches!(days, 7 | 30 | 90) {
            return Ok(Vec::new());
        }

        let api_key_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM api_keys WHERE LOWER(wallet_address) = LOWER($1)",
        )
        .bind(wallet_address)
        .fetch_all(&self.core_pool)
        .await?;

        if api_key_ids.is_empty() {
            return Ok(Vec::new());
        }

        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(i64::from(days - 1));

        let endpoints: Vec<TopEndpoint> = sqlx::query_as(
            r#"
            SELECT endpoint::text, method::text,
                   COUNT(*)::bigint AS count
            FROM infra_logs.api_key_usage_logs
            WHERE api_key_id = ANY($1)
              AND request_at >= $2::date
            GROUP BY endpoint, method
            ORDER BY count DESC
            LIMIT $3
            "#,
        )
        .bind(&api_key_ids)
        .bind(start_date)
        .bind(limit)
        .fetch_all(&self.analytics_pool)
        .await?;

        Ok(endpoints)
    }

    pub async fn get_module_usage(
        &self,
        _wallet_address: &str,
        _days: i32,
    ) -> Result<Vec<ModuleUsage>, sqlx::Error> {
        // TODO: implement via JOIN against api_modules + usage logs.
        Ok(Vec::new())
    }
}