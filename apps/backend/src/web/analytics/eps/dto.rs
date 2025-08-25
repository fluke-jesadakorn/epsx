// DTOs and Request/Response Structures for EPS Analytics
// Focused module handling all data transfer objects and API structures

use serde::{Deserialize, Serialize};
use crate::dom::services::eps_cache_service::CacheStats;

/// Query parameters for EPS rankings endpoint
#[derive(Debug, Deserialize)]
pub struct EPSRankingQueryParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// API response structure matching frontend pattern
#[derive(Debug, Serialize)]
pub struct EPSRankingsApiResponse {
    pub data: Vec<crate::dom::entities::eps_growth::EPSRanking>,
    pub pagination: EPSPaginationResponse,
}

/// Pagination response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct EPSPaginationResponse {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

/// Country data with display name and API value
#[derive(Debug, Serialize)]
pub struct CountryData {
    pub value: String,      // API value (lowercase)
    pub label: String,      // Display name (proper capitalization)
}

/// Countries list response  
#[derive(Debug, Serialize)]
pub struct CountriesResponse {
    pub countries: Vec<CountryData>,
    pub count: usize,
}

/// Sectors list response
#[derive(Debug, Serialize)]
pub struct SectorsResponse {
    pub sectors: Vec<String>,
    pub count: usize,
    pub country: Option<String>,
}

/// Health check response for EPS service
#[derive(Debug, Serialize)]
pub struct EPSHealthResponse {
    pub status: String,
    pub message: String,
    pub available_countries: usize,
}

/// Unified analytics rankings response structure
#[derive(Debug, Serialize)]
pub struct UnifiedAnalyticsRankingsResponse {
    pub success: bool,
    pub data: Vec<UnifiedRankingItem>,
    pub pagination: EPSPaginationResponse,
    pub metadata: UnifiedAnalyticsMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual ranking item in unified format
#[derive(Debug, Serialize)]
pub struct UnifiedRankingItem {
    pub symbol: String,
    pub company_name: String,
    pub ranking_position: i32,
    pub current_price: f64,
    pub current_price_date: chrono::DateTime<chrono::Utc>,
    pub quarterly_data: Vec<QuarterlyData>,
    pub market_data: MarketData,
    pub analytics: AnalyticsMetrics,
}

/// Quarterly data for each stock
#[derive(Debug, Serialize)]
pub struct QuarterlyData {
    pub quarter: String, // e.g., "Q3 '25"
    pub date: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64, // QoQ growth percentage
    pub price_growth: f64, // QoQ price growth percentage
    pub volume: Option<i64>,
}

/// Market data for each stock
#[derive(Debug, Serialize)]
pub struct MarketData {
    pub market_cap: Option<i64>,
    pub volume_24h: Option<i64>,
    pub country: String,
    pub sector: String,
    pub exchange: String,
}

/// Analytics metrics for each stock
#[derive(Debug, Serialize)]
pub struct AnalyticsMetrics {
    pub qoq_growth: f64,
    pub ranking_score: f64,
    pub trend: String, // bullish, bearish, neutral, etc.
    pub volatility: f64,
}

/// Metadata included in unified response
#[derive(Debug, Serialize)]
pub struct UnifiedAnalyticsMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub current_filters: UnifiedFilters,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
    pub enhanced_with_websocket: bool,
}

/// Current filters applied to the request
#[derive(Debug, Serialize)]
pub struct UnifiedFilters {
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: String,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// Cache statistics response
#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub success: bool,
    pub stats: CacheStats,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache refresh response
#[derive(Debug, Serialize)]
pub struct CacheRefreshResponse {
    pub success: bool,
    pub refreshed_entries: usize,
    pub duration_ms: u64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache health check response
#[derive(Debug, Serialize)]
pub struct CacheHealthResponse {
    pub status: String,
    pub healthy: bool,
    pub cache_stats: CacheStats,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Card dashboard response structure for multi-symbol EPS analytics
#[derive(Debug, Serialize, Deserialize)]
pub struct CardDashboardResponse {
    pub success: bool,
    pub data: Vec<SymbolCardData>,
    pub pagination: EPSPaginationResponse,
    pub metadata: CardDashboardMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual symbol card data matching frontend UI requirements
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolCardData {
    pub rank: i32,
    pub symbol: String,
    pub latest_date: String,
    pub value: f64,                    // Current price
    pub active_status: String,         // Active or Non Active based on surplus
    pub quarterly_performance: Vec<QuarterlyPerformanceData>,
}

/// Quarterly performance data for the card dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarterlyPerformanceData {
    pub quarter: String,      // "Q1", "Q0", etc.
    pub date: String,         // "Aug 8, 2025"
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,      // EPS % growth
    pub price_growth: f64,    // Price % growth
}

/// Metadata for card dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct CardDashboardMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
}

impl From<crate::dom::entities::eps_growth::EPSRankingsResponse> for EPSRankingsApiResponse {
    fn from(response: crate::dom::entities::eps_growth::EPSRankingsResponse) -> Self {
        Self {
            data: response.rankings,
            pagination: EPSPaginationResponse {
                page: response.pagination.page,
                limit: response.pagination.limit,
                total: response.pagination.total,
                total_pages: response.pagination.total_pages,
                has_next: response.pagination.has_next,
                has_prev: response.pagination.has_prev,
            },
        }
    }
}