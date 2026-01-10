use std::sync::Arc;
use crate::config::Config;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use super::types::{TradingViewConfig, MarketDataError, FrontendEPSResponse};
use super::rest::TradingViewRestClient;
use super::scanner::TradingViewScanner;
use super::websocket::TradingViewWebSocketHandler;

/// Main API service for TradingView integration
/// Aggregates REST, Scanner, and WebSocket capabilities
pub struct TradingViewApiService {
    pub rest_client: TradingViewRestClient,
    pub scanner: TradingViewScanner,
    pub websocket: TradingViewWebSocketHandler,
    pub cache: std::sync::Arc<tokio::sync::RwLock<super::cache::TradingViewCache>>,
    pub config: TradingViewConfig,
}

impl TradingViewApiService {
    /// Create new API service from configuration
    pub fn new(config: Arc<Config>) -> Self {
        let tv_config = TradingViewConfig::from(&*config);
        
        Self {
            rest_client: TradingViewRestClient::new(tv_config.clone()),
            scanner: TradingViewScanner::new(tv_config.clone()),
            websocket: TradingViewWebSocketHandler::new(tv_config.clone()),
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(super::cache::TradingViewCache::new())),
            config: tv_config,
        }
    }

    /// Fetch screening data via scanner
    pub async fn fetch_eps_growth_ranking(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> Result<(Vec<StockScreeningResult>, i32), MarketDataError> {
        let payload = self.scanner.build_screener_request_with_params(
            skip.unwrap_or(0),
            limit.unwrap_or(10),
            country,
            sector,
            sort_by,
        );
        
        let response = self.rest_client.execute_custom_request(payload, 3).await?;
        let total = response.total_count.unwrap_or(response.data.len() as i32);
        let results = self.scanner.process_trading_view_response(response);
        
        Ok((results, total))
    }

    /// Fetch specific symbols concurrently
    pub async fn fetch_symbols_concurrent(&self, symbols: Vec<String>) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        let payload = self.scanner.build_symbols_request(symbols);
        let response = self.rest_client.execute_custom_request(payload, 3).await?;
        let screening_results = self.scanner.process_trading_view_response(response);
        
        // Convert screening results to EPS growth data (simplified conversion for now)
        let eps_data = screening_results.into_iter().map(|s| {
            EPSGrowthData {
                symbol: s.symbol,
                name: s.name,
                country: "unknown".to_string(), // Field not in StockScreeningResult
                sector: s.sector.unwrap_or_else(|| "unknown".to_string()),
                exchange: "unknown".to_string(), // Field not in StockScreeningResult
                current_eps: s.current_eps,
                growth_factor: s.eps_growth_yoy,
                price_current: Some(s.price),
                market_cap: s.market_cap.map(|m| m as i64),
                volume: Some(s.volume as i64),
                ranking_score: None,
                created_at: None,
                updated_at: None,
                next_earnings_date: s.next_earnings_date.map(|d| d.to_string()),
                last_earnings_date: s.last_earnings_date.map(|d| d.to_string()),
            }
        }).collect();
        
        Ok(eps_data)
    }

    /// Test connections for health check
    pub async fn test_connections(&self) -> Result<bool, MarketDataError> {
        self.rest_client.test_connection().await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> super::cache::CacheStats {
        let cache = self.cache.read().await;
        cache.get_stats()
    }

    /// Clear all cache entries
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear_all();
    }
}
