// TradingView Types - Focused Module for Data Structures and DTOs
// Contains all type definitions, request/response structures, and configuration

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::config::Config;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;

/// TradingView-specific error types
#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("External API error: {0}")]
    ExternalApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// TradingView API response structure (matches actual API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingViewResponse {
    pub data: Vec<TradingViewStock>,
    /// Total count is not returned by TradingView API, calculated from data length
    #[serde(default, rename = "totalCount")]
    pub total_count: Option<i32>,
}

/// TradingView stock data from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingViewStock {
    pub s: String, // Symbol
    pub d: Vec<StockDataField>, // Data array
}

/// TradingView stock data field enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StockDataField {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<serde_json::Value>),
    Object(serde_json::Map<String, serde_json::Value>),
    Null,
}

/// Phase information for stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInfo {
    pub date: String,
    pub active: bool,
}

/// Phase status for stock analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseStatus {
    pub date: String,
    pub phase_type: PhaseType,
    pub active: bool,
}

/// Phase type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseType {
    Buy,
    Sell, 
    Monitor,
}

/// Frontend EPS data structure (matches exact frontend format)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendEPSData {
    pub id: String,
    pub symbol: String,
    pub company_name: String,
    pub current_eps: f64,
    pub qoq_growth: f64,
    pub market_cap: i64,
    pub price_current: f64,
    pub volume: i64,
    pub country: String,
    pub sector: String,
    pub ranking_score: f64,
}

/// Frontend pagination structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendPagination {
    pub page: i32,
    pub limit: i32,
    pub total: i32,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

/// Complete frontend response structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontendEPSResponse {
    pub data: Vec<FrontendEPSData>,
    pub pagination: FrontendPagination,
}

/// TradingView integration configuration
#[derive(Debug, Clone)]
pub struct TradingViewConfig {
    pub scanner_api_url: String,
    pub websocket_url: String,
    pub origin_url: String,
    pub referer_url: String,
    pub http_timeout_seconds: u64,
    pub auth_token: String,
}

impl From<&Config> for TradingViewConfig {
    fn from(config: &Config) -> Self {
        Self {
            scanner_api_url: "https://scanner.tradingview.com/global/scan?label-product=screener-stock".to_string(),
            websocket_url: "wss://data.tradingview.com/socket.io/websocket".to_string(),
            origin_url: "https://www.tradingview.com".to_string(),
            referer_url: "https://www.tradingview.com/".to_string(),
            http_timeout_seconds: config.external_services.tradingview.http_timeout_seconds,
            auth_token: config.auth.firebase_project_id.clone().unwrap_or_else(|| "default-project".to_string()),
        }
    }
}

/// TradingView service port trait for domain layer
#[async_trait]
pub trait TradingViewService: Send + Sync {
    /// Fetch stock screener data
    async fn fetch_screener_data(&self) -> Result<Vec<StockScreeningResult>, MarketDataError>;
    
    /// Connect to TradingView WebSocket for real-time data
    async fn connect_realtime_feed(&self) -> Result<(), MarketDataError>;
    
    /// Fetch EPS growth ranking data with server-side pagination and filtering
    async fn fetch_eps_growth_ranking(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> Result<(Vec<StockScreeningResult>, i32), MarketDataError>;
    
    /// Extract EPS growth data from TradingView response
    async fn extract_eps_growth_data(&self) -> Result<Vec<EPSGrowthData>, MarketDataError>;
    
    /// Extract EPS growth data with concurrent batch processing
    async fn extract_eps_growth_data_concurrent(&self, batch_size: usize) -> Result<Vec<EPSGrowthData>, MarketDataError>;
    
    /// Fetch specific symbols concurrently
    async fn fetch_symbols_concurrent(&self, symbols: Vec<String>) -> Result<Vec<EPSGrowthData>, MarketDataError>;
    
    /// Fetch EPS data in frontend format with pagination
    async fn fetch_eps_rankings_for_frontend(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
    ) -> Result<FrontendEPSResponse, MarketDataError>;
    
    /// Fetch enhanced EPS data with WebSocket details
    async fn fetch_enhanced_eps_rankings(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        use_websocket: bool,
    ) -> Result<FrontendEPSResponse, MarketDataError>;
}

/// Type aliases for common data transformations
pub type StockDataResult<T> = Result<T, String>;
pub type EPSDataBatch = Vec<EPSGrowthData>;
pub type FrontendDataBatch = Vec<FrontendEPSData>;

/// Constants for TradingView API integration
pub mod constants {
    pub const DEFAULT_PAGE_SIZE: i32 = 10;
    pub const MAX_PAGE_SIZE: i32 = 100;
    pub const MAX_CONCURRENT_REQUESTS: usize = 5;
    pub const BATCH_DELAY_MS: u64 = 500;
    pub const MAX_EPS_VALUE: f64 = 50000.0;
    pub const MIN_EPS_VALUE: f64 = 0.001;
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_frontend_eps_data_creation() {
        let data = FrontendEPSData {
            id: Uuid::new_v4().to_string(),
            symbol: "AAPL".to_string(),
            company_name: "Apple Inc".to_string(),
            current_eps: 3.25,
            qoq_growth: 12.5,
            market_cap: 2500000000000,
            price_current: 150.0,
            volume: 50000000,
            country: "america".to_string(),
            sector: "Technology".to_string(),
            ranking_score: 85.5,
        };

        assert_eq!(data.symbol, "AAPL");
        assert_eq!(data.current_eps, 3.25);
        assert!(data.qoq_growth > 0.0);
    }

    #[test]
    fn test_frontend_pagination() {
        let pagination = FrontendPagination {
            page: 1,
            limit: 10,
            total: 100,
            total_pages: 10,
            has_next: true,
            has_prev: false,
        };

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.total_pages, 10);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);
    }

    #[test]
    fn test_tradingview_config_defaults() {
        use crate::config::*;
        
        let config = Config::default();
        let tv_config = TradingViewConfig::from(&config);
        
        assert!(tv_config.scanner_api_url.contains("scanner.tradingview.com"));
        assert!(tv_config.websocket_url.contains("wss://"));
        assert_eq!(tv_config.origin_url, "https://www.tradingview.com");
    }

    #[test]
    fn test_constants_values() {
        use super::constants::*;
        
        assert_eq!(DEFAULT_PAGE_SIZE, 10);
        assert_eq!(MAX_PAGE_SIZE, 100);
        assert_eq!(MAX_CONCURRENT_REQUESTS, 5);
        assert!(MAX_EPS_VALUE > MIN_EPS_VALUE);
    }
}