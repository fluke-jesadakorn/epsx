// DTOs and Request/Response Structures for EPS Analytics
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

/// Access information for rank-based permissions
#[derive(Debug, Serialize, Clone)]
pub struct AccessInfo {
    pub min_accessible_rank: i32,  // Minimum rank user can access
    pub locked_ranks_count: i32,    // How many ranks are locked (same as min_accessible_rank - 1)
}

/// API response structure matching frontend pattern
#[derive(Debug, Serialize)]
pub struct EPSRankingsApiResponse {
    pub data: Vec<crate::domain::shared_kernel::entities::eps_growth::EPSRanking>,
    pub pagination: EPSPaginationResponse,
    pub access_info: AccessInfo,  // User's access level information
}

/// Pagination response structure
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
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
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CountryData {
    pub value: String,      // API value (lowercase)
    pub label: String,      // Display name (proper capitalization)
}

/// Countries list response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CountriesResponse {
    pub countries: Vec<CountryData>,
    pub count: usize,
}

/// Sectors list response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SectorsResponse {
    pub sectors: Vec<String>,
    pub count: usize,
    pub country: Option<String>,
}

/// Combined filters response for frontend
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct FiltersResponse {
    pub countries: Vec<CountryData>,
    pub sectors: Vec<String>,
    pub exchanges: Vec<String>,
    pub stock_types: Vec<String>,
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
    // Real earnings announcement dates from TradingView (Unix timestamps)
    pub next_earnings_date: Option<i64>,
    pub last_earnings_date: Option<i64>,
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
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CacheStatsResponse {
    pub success: bool,
    pub stats: CacheStats,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache refresh response
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CacheRefreshResponse {
    pub success: bool,
    pub refreshed_entries: usize,
    pub duration_ms: u64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Card dashboard response structure for multi-symbol EPS analytics
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CardDashboardResponse {
    pub success: bool,
    pub data: Vec<SymbolCardData>,
    pub pagination: EPSPaginationResponse,
    pub metadata: CardDashboardMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual symbol card data matching frontend UI requirements
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SymbolCardData {
    pub rank: i32,
    pub symbol: String,
    pub latest_date: String,
    pub value: f64,                    // Current price
    pub active_status: String,         // Active or Non Active based on surplus
    pub quarterly_performance: Vec<QuarterlyPerformanceData>,
    pub next_quarter_estimate: Option<NextQuarterEstimate>, // NEW: Next quarter EPS estimate
    pub eps_quarterly: Option<EPSQuarterlyData>, // NEW: 4-Quarter EPS data structure
    pub next_earnings_date: Option<i64>, // Unix timestamp from TradingView (raw)
    pub last_earnings_date: Option<i64>, // Unix timestamp from TradingView (raw)
    // Frontend-ready formatted fields (calculated by backend)
    pub next_earnings_date_formatted: Option<String>, // "Nov 18, 2025"
    pub days_until_next_earnings: Option<i32>,        // 185
    pub progress_percentage: Option<f64>,             // 0-100 for progress bar
    // Top-level fields for frontend (derived from quarterly_performance[0])
    pub current_eps: Option<f64>,      // From quarterly_performance[0].eps
    pub growth_factor: Option<f64>,    // From quarterly_performance[0].eps_growth
    pub price_current: Option<f64>,    // From quarterly_performance[0].price
}

/// 4-Quarter EPS Data Structure matching frontend expectations
#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EPSQuarterlyData {
    pub eps_q_minus_2: Option<f64>,        // Q-2 (2 quarters ago)
    pub eps_q_minus_1: Option<f64>,        // Q-1 (1 quarter ago) 
    pub eps_q_current: Option<f64>,        // Q0 (current quarter)
    pub eps_q_next_estimate: Option<f64>,  // Q+1 (next quarter estimate)
    
    // Quarter dates for EPS reporting
    pub eps_q_minus_2_date: Option<String>,
    pub eps_q_minus_1_date: Option<String>, 
    pub eps_q_current_date: Option<String>,
    pub eps_q_next_estimate_date: Option<String>,
    
    // Growth calculations
    pub qoq_growth_current: Option<f64>,   // Q0 vs Q-1 growth percentage
    pub yoy_growth_current: Option<f64>,   // Q0 vs Q-4 growth (if available)
    pub trend_direction: Option<String>,   // "UP", "DOWN", "FLAT"
    pub avg_growth_rate: Option<f64>,      // Average growth rate across available quarters
    pub consistency_score: Option<String>, // "HIGH", "MEDIUM", "LOW" - earnings consistency
}

/// Quarterly performance data for the card dashboard
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
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
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CardDashboardMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
}

/// Convert from DDD market analytics to API response
impl EPSRankingsApiResponse {
    pub fn from_ddd_ranking_entry(ranking_entry: crate::domain::market_analytics::aggregates::eps_ranking::RankingEntry, rank: u32, page: i32, limit: i32, total: i64) -> Self {
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
            access_info: AccessInfo {
                min_accessible_rank: 0,  // Default: full access for DDD rankings
                locked_ranks_count: 0,
            },
        }
    }
    
    /// Convert DDD RankingEntry to legacy EPSRanking for API compatibility
    fn convert_ddd_entry_to_legacy_ranking(
        entry: crate::domain::market_analytics::aggregates::eps_ranking::RankingEntry, 
        rank: u32
    ) -> crate::domain::shared_kernel::entities::eps_growth::EPSRanking {
        crate::domain::shared_kernel::entities::eps_growth::EPSRanking::from_eps_data(
            crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData {
                symbol: entry.symbol.to_string(),
                name: entry.company_name,
                country: entry.country.name().to_string(),
                sector: entry.sector.to_string(),
                exchange: "NASDAQ".to_string(), // Default exchange
                current_eps: Some(entry.eps_value.value()),
                growth_factor: Some(entry.growth_factor.percentage()),
                price_current: None, // Not available from entry
                market_cap: None, // Would need to be calculated or provided
                volume: None, // Not available from entry
                ranking_score: Some(rank as f64), // Use rank as score
                created_at: None,
                updated_at: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            Some(rank as i32)
        )
    }
}