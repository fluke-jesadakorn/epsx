use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult};
use crate::application::trading_analytics::queries::{
    GetSystemMetricsQuery, GetSystemMetricsResponse, CacheMetrics, DatabaseMetrics, ApiMetrics,
};
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};

/// Query handler for getting multi-source system metrics
pub struct GetSystemMetricsQueryHandler {
    tradingview_service: Arc<TradingViewApiService>,
    db_pool: Arc<&'static Pool<AsyncPgConnection>>,
}

impl GetSystemMetricsQueryHandler {
    pub fn new(tradingview_service: Arc<TradingViewApiService>, db_pool: Arc<&'static Pool<AsyncPgConnection>>) -> Self {
        Self {
            tradingview_service,
            db_pool,
        }
    }

    /// Collect cache metrics from TradingView cache
    async fn collect_cache_metrics(&self) -> Option<CacheMetrics> {
        let cache_stats = self.tradingview_service.get_cache_stats();

        Some(CacheMetrics {
            status: "healthy".to_string(),
            hit_rate: cache_stats.hit_ratio,
            entry_count: cache_stats.total_count,
            total_size_kb: cache_stats.total_count * 10, // Rough estimate
        })
    }

    /// Collect database metrics from connection pool
    async fn collect_database_metrics(&self) -> Option<DatabaseMetrics> {
        let status = self.db_pool.status();
        let pool_size = status.max_size;
        let available = status.available as usize;
        let active = pool_size.saturating_sub(available);

        Some(DatabaseMetrics {
            status: "healthy".to_string(),
            connection_pool_size: pool_size as i32,
            active_connections: active as i32,
            idle_connections: available as i32,
            avg_query_time_ms: 5.0, // Placeholder - would need query monitoring
        })
    }

    /// Collect API metrics from external services
    async fn collect_api_metrics(&self) -> Option<ApiMetrics> {
        // Test TradingView connection
        let start = std::time::Instant::now();
        let tv_status = match self.tradingview_service.test_connections().await {
            Ok(_) => "healthy",
            Err(_) => "degraded",
        };
        let tv_response_time = start.elapsed().as_millis() as u64;

        Some(ApiMetrics {
            tradingview_status: tv_status.to_string(),
            tradingview_response_time_ms: tv_response_time,
            websocket_status: "healthy".to_string(),
            websocket_connections: 0, // Would need WebSocket service integration
        })
    }

    /// Calculate overall system health based on component metrics
    fn calculate_overall_health(
        cache: &Option<CacheMetrics>,
        database: &Option<DatabaseMetrics>,
        api: &Option<ApiMetrics>,
    ) -> String {
        let mut health_score = 0;
        let mut total_components = 0;

        if let Some(cache_m) = cache {
            total_components += 1;
            if cache_m.status == "healthy" {
                health_score += 1;
            }
        }

        if let Some(db_m) = database {
            total_components += 1;
            if db_m.status == "healthy" {
                health_score += 1;
            }
        }

        if let Some(api_m) = api {
            total_components += 1;
            if api_m.tradingview_status == "healthy" && api_m.websocket_status == "healthy" {
                health_score += 1;
            }
        }

        if total_components == 0 {
            return "unknown".to_string();
        }

        let health_percentage = (health_score as f64 / total_components as f64) * 100.0;

        match health_percentage {
            p if p >= 90.0 => "healthy".to_string(),
            p if p >= 70.0 => "degraded".to_string(),
            _ => "unhealthy".to_string(),
        }
    }
}

#[async_trait]
impl QueryHandler<GetSystemMetricsQuery> for GetSystemMetricsQueryHandler {
    async fn handle(
        &self,
        query: GetSystemMetricsQuery,
    ) -> ApplicationResult<GetSystemMetricsResponse> {
        // Collect metrics based on query flags
        let cache_metrics = if query.include_cache.unwrap_or(true) {
            self.collect_cache_metrics().await
        } else {
            None
        };

        let database_metrics = if query.include_database.unwrap_or(true) {
            self.collect_database_metrics().await
        } else {
            None
        };

        let api_metrics = if query.include_external_apis.unwrap_or(true) {
            self.collect_api_metrics().await
        } else {
            None
        };

        // Calculate overall health
        let overall_health =
            Self::calculate_overall_health(&cache_metrics, &database_metrics, &api_metrics);

        Ok(GetSystemMetricsResponse {
            success: true,
            timestamp: chrono::Utc::now(),
            cache_metrics,
            database_metrics,
            api_metrics,
            overall_health,
        })
    }
}
