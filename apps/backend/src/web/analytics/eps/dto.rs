use chrono::{DateTime, Utc};// DTOs and Request/Response Structures for EPS Analytics
// Focused module handling all data transfer objects and API structures

use serde::{Deserialize, Serialize};
use crate::domain::shared_kernel::services::eps_cache_service::CacheStats;

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
    pub data: Vec<crate::domain::shared_kernel::entities::eps_growth::EPSRanking>,
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
    // Real earnings announcement dates from TradingView
    pub next_earnings_date: Option<String>,
    pub last_earnings_date: Option<String>,
}

/// Quarterly data for each stock
#[derive(Debug, Serialize, Clone)]
pub struct QuarterlyData {
    pub quarter: String, // e.g., "Q3 '25"
    pub date: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64, // Growth factor percentage
    pub price_growth: f64, // Price growth percentage
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
    pub growth_factor: f64,
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
    pub next_quarter_estimate: Option<NextQuarterEstimate>, // NEW: Next quarter EPS estimate
}

/// Quarterly performance data for the card dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarterlyPerformanceData {
    pub quarter: String,      // "Q1", "Q0", etc. OR "Announced Jul 25, 2024" 
    pub date: String,         // "Aug 8, 2025"
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,      // EPS % growth
    pub price_growth: f64,    // Price % growth
    // NEW: Enhanced announcement date fields
    pub announcement_date: Option<String>,     // "Est. Oct 24, 2025" or "Announced Jul 25, 2024"
    pub announcement_timestamp: Option<i64>,   // Raw timestamp for calculations
    pub is_estimated: bool,                    // true if future/estimated, false if past/announced
}

/// Next quarter EPS estimate data
#[derive(Debug, Serialize, Deserialize)]
pub struct NextQuarterEstimate {
    pub quarter: String,                       // "2025-Q4"
    pub estimated_eps: f64,                    // 3.85
    pub announcement_date: String,             // "Est. Oct 24, 2025"
    pub announcement_timestamp: i64,           // Raw timestamp
    pub days_until_announcement: i32,          // 45 (calculated days from now)
    pub estimated_price_target: Option<f64>,  // Optional price target based on EPS
    pub confidence: String,                    // "High", "Medium", "Low" based on data quality
}

/// Metadata for card dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct CardDashboardMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
}

/// Convert from DDD trading analytics to API response
impl EPSRankingsApiResponse {
    pub fn from_ddd_ranking_entry(ranking_entry: crate::domain::trading_analytics::aggregates::eps_ranking::RankingEntry, rank: u32, page: i32, limit: i32, total: i64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as i32;
        
        // Convert DDD RankingEntry to legacy EPSRanking for API compatibility
        let legacy_ranking = Self::convert_ddd_entry_to_legacy_ranking(ranking_entry, rank);
        
        Self {
            data: vec![legacy_ranking],
            pagination: EPSPaginationResponse {
                page,
                limit,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }
    
    /// Convert DDD RankingEntry to legacy EPSRanking for API compatibility
    fn convert_ddd_entry_to_legacy_ranking(
        entry: crate::domain::trading_analytics::aggregates::eps_ranking::RankingEntry, 
        rank: u32
    ) -> crate::domain::shared_kernel::entities::eps_growth::EPSRanking {
        crate::domain::shared_kernel::entities::eps_growth::EPSRanking {
            symbol: entry.symbol.to_string(),
            company_name: entry.company_name,
            eps_current: entry.eps_value.value(),
            eps_previous: 0.0, // Not available from entry
            growth_rate: entry.growth_factor.percentage(),
            rank,
            sector: entry.sector.to_string(),
            market_cap: None, // Would need to be calculated or provided
            price_current: None, // Not available from entry
            last_updated: chrono::Utc::now(),
        }
    }
}