// Get System Metrics Query
// Multi-source system health and performance metrics for admin dashboard

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;

#[derive(Debug, Clone)]
pub struct GetSystemMetricsQuery {
    pub include_cache: Option<bool>,
    pub include_database: Option<bool>,
    pub include_external_apis: Option<bool>,
}

impl Query for GetSystemMetricsQuery {
    type Response = GetSystemMetricsResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSystemMetricsResponse {
    pub success: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cache_metrics: Option<CacheMetrics>,
    pub database_metrics: Option<DatabaseMetrics>,
    pub api_metrics: Option<ApiMetrics>,
    pub overall_health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub status: String,
    pub hit_rate: f64,
    pub entry_count: usize,
    pub total_size_kb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub status: String,
    pub connection_pool_size: i32,
    pub active_connections: i32,
    pub idle_connections: i32,
    pub avg_query_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub tradingview_status: String,
    pub tradingview_response_time_ms: u64,
    pub websocket_status: String,
    pub websocket_connections: i32,
}
