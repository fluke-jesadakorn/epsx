// Response DTOs for Trading Analytics
// Moved from web/analytics/eps/types.rs following Clean Architecture

use serde::{Deserialize, Serialize};
use crate::domain::shared_kernel::services::eps_cache_service::CacheStats;

// ============================================================================
// ACCESS & PAGINATION
// ============================================================================

/// Access information for rank-based permissions
#[derive(Debug, Serialize, Clone)]
pub struct AccessInfo {
    pub min_accessible_rank: i32,
    pub locked_ranks_count: i32,
}

/// Pagination response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ============================================================================
// EPS RANKINGS RESPONSES
// ============================================================================

/// API response structure for EPS rankings
#[derive(Debug, Serialize)]
pub struct EPSRankingsApiResponse {
    pub data: Vec<crate::domain::shared_kernel::entities::eps_growth::EPSRanking>,
    pub pagination: EPSPaginationResponse,
    pub access_info: AccessInfo,
}

/// Unified analytics rankings response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAnalyticsRankingsResponse {
    pub success: bool,
    pub data: Vec<UnifiedRankingItem>,
    pub pagination: EPSPaginationResponse,
    pub metadata: UnifiedAnalyticsMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual ranking item in unified format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRankingItem {
    pub symbol: String,
    pub company_name: String,
    pub ranking_position: i32,
    pub current_price: f64,
    pub current_price_date: chrono::DateTime<chrono::Utc>,
    pub quarterly_data: Vec<QuarterlyData>,
    pub market_data: MarketData,
    pub analytics: AnalyticsMetrics,
    pub next_earnings_date: Option<i64>,
    pub last_earnings_date: Option<i64>,
}

/// Quarterly data for each stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyData {
    pub quarter: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,
    pub price_growth: f64,
    pub volume: Option<i64>,
}

/// Market data for each stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub market_cap: Option<i64>,
    pub volume_24h: Option<i64>,
    pub country: String,
    pub sector: String,
    pub exchange: String,
}

/// Analytics metrics for each stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub growth_factor: f64,
    pub ranking_score: f64,
    pub trend: String,
    pub volatility: f64,
}

/// Metadata included in unified response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedAnalyticsMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub current_filters: super::UnifiedFilters,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
    pub enhanced_with_websocket: bool,
}

// ============================================================================
// METADATA RESPONSES
// ============================================================================

/// Country data with display name and API value
#[derive(Debug, Serialize)]
pub struct CountryData {
    pub value: String,
    pub label: String,
}

/// Countries list response
#[derive(Debug, Serialize)]
pub struct CountriesResponse {
    pub countries: Vec<CountryData>,
    pub count: usize,
}

/// Sectors list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorsResponse {
    pub sectors: Vec<String>,
    pub count: usize,
    pub country: Option<String>,
}

/// Combined filters response for frontend
#[derive(Debug, Serialize)]
pub struct FiltersResponse {
    pub countries: Vec<CountryData>,
    pub sectors: Vec<String>,
    pub exchanges: Vec<String>,
    pub stock_types: Vec<String>,
}

// ============================================================================
// HEALTH & CACHE RESPONSES
// ============================================================================

/// Health check response for EPS service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSHealthResponse {
    pub status: String,
    pub message: String,
    pub available_countries: usize,
}

/// Cache statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealthResponse {
    pub status: String,
    pub healthy: bool,
    pub cache_stats: CacheStats,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// CARD DASHBOARD RESPONSES
// ============================================================================

/// Card dashboard response structure for multi-symbol EPS analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDashboardResponse {
    pub success: bool,
    pub data: Vec<SymbolCardData>,
    pub pagination: EPSPaginationResponse,
    pub metadata: CardDashboardMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual symbol card data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolCardData {
    pub rank: i32,
    pub symbol: String,
    pub latest_date: String,
    pub value: f64,
    pub active_status: String,
    pub quarterly_performance: Vec<QuarterlyPerformanceData>,
    pub next_quarter_estimate: Option<NextQuarterEstimate>,
    pub eps_quarterly: Option<EPSQuarterlyData>,
    pub next_earnings_date: Option<i64>,
    pub last_earnings_date: Option<i64>,
    pub next_earnings_date_formatted: Option<String>,
    pub days_until_next_earnings: Option<i32>,
    pub progress_percentage: Option<f64>,
}

/// 4-Quarter EPS Data Structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EPSQuarterlyData {
    pub eps_q_minus_2: Option<f64>,
    pub eps_q_minus_1: Option<f64>,
    pub eps_q_current: Option<f64>,
    pub eps_q_next_estimate: Option<f64>,
    pub eps_q_minus_2_date: Option<String>,
    pub eps_q_minus_1_date: Option<String>,
    pub eps_q_current_date: Option<String>,
    pub eps_q_next_estimate_date: Option<String>,
    pub qoq_growth_current: Option<f64>,
    pub yoy_growth_current: Option<f64>,
    pub trend_direction: Option<String>,
    pub avg_growth_rate: Option<f64>,
    pub consistency_score: Option<String>,
}

/// Quarterly performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyPerformanceData {
    pub quarter: String,
    pub date: String,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,
    pub price_growth: f64,
    pub announcement_date: Option<String>,
    pub announcement_timestamp: Option<i64>,
    pub is_estimated: bool,
}

/// Next quarter EPS estimate data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextQuarterEstimate {
    pub quarter: String,
    pub estimated_eps: f64,
    pub announcement_date: String,
    pub announcement_timestamp: i64,
    pub days_until_announcement: i32,
    pub estimated_price_target: Option<f64>,
    pub confidence: String,
}

/// Metadata for card dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDashboardMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
}

// ============================================================================
// HELPER IMPLEMENTATIONS
// ============================================================================

impl EPSRankingsApiResponse {
    pub fn from_ddd_ranking_entry(
        ranking_entry: crate::domain::market_analytics::aggregates::eps_ranking::RankingEntry,
        rank: u32,
        page: i32,
        limit: i32,
        total: i64,
    ) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as i32;
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
            access_info: AccessInfo {
                min_accessible_rank: 0,
                locked_ranks_count: 0,
            },
        }
    }

    fn convert_ddd_entry_to_legacy_ranking(
        entry: crate::domain::market_analytics::aggregates::eps_ranking::RankingEntry,
        rank: u32,
    ) -> crate::domain::shared_kernel::entities::eps_growth::EPSRanking {
        crate::domain::shared_kernel::entities::eps_growth::EPSRanking::from_eps_data(
            crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData {
                symbol: entry.symbol.to_string(),
                name: entry.company_name,
                country: entry.country.name().to_string(),
                sector: entry.sector.to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(entry.eps_value.value()),
                growth_factor: Some(entry.growth_factor.percentage()),
                price_current: None,
                market_cap: None,
                volume: None,
                ranking_score: Some(rank as f64),
                created_at: None,
                updated_at: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            Some(rank as i32),
        )
    }
}
