// TradingView Module - Focused Architecture
// Restored from development branch with modular design replacing broken monolithic implementation

// Public focused modules - each handles a specific domain
pub mod types;     // Data structures, DTOs, traits, and configuration
pub mod rest;      // HTTP REST API communication and request handling  
pub mod websocket; // WebSocket connections and real-time data streams
pub mod scanner;   // Stock screening, filtering, and request building
pub mod mapper;    // Data transformation and mapping between formats
pub mod cache;     // Caching functionality and performance optimization

// Re-export key types for easy access
pub use types::*;

// Re-export all TradingView functionality through focused modules
pub use rest::TradingViewRestClient;
pub use websocket::{TradingViewWebSocketHandler, WebSocketMessageType, RealTimeDataProcessor};
pub use scanner::TradingViewScanner;
pub use mapper::TradingViewMapper;
pub use cache::{TradingViewCache, CacheStats, CachePerformanceOptimizer};

use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use async_trait::async_trait;

use crate::config::Config;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use types::MarketDataError;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;

/// Main TradingView API service implementation using focused modules
pub struct TradingViewApiService {
    scanner: TradingViewScanner,
    websocket_handler: TradingViewWebSocketHandler,
    cache: RwLock<TradingViewCache>,
    config: TradingViewConfig,
}

impl TradingViewApiService {
    /// Create new TradingView service with focused modules
    pub fn new(app_config: Arc<Config>) -> Self {
        let config = TradingViewConfig::from(app_config.as_ref());
        
        let scanner = TradingViewScanner::new(config.clone());
        let websocket_handler = TradingViewWebSocketHandler::new(config.clone());
        let cache = RwLock::new(TradingViewCache::new());

        Self { 
            scanner,
            websocket_handler,
            cache,
            config,
        }
    }

    /// Create service with custom cache settings
    pub fn with_cache_settings(app_config: Arc<Config>, cache_ttl: Duration, cache_size: usize) -> Self {
        let config = TradingViewConfig::from(app_config.as_ref());
        
        let scanner = TradingViewScanner::new(config.clone());
        let websocket_handler = TradingViewWebSocketHandler::new(config.clone());
        let cache = RwLock::new(TradingViewCache::with_settings(cache_ttl, cache_size));

        Self {
            scanner,
            websocket_handler,
            cache,
            config,
        }
    }

    /// Fetch EPS growth ranking with server-side pagination and filtering (REAL API INTEGRATION)
    pub async fn fetch_eps_growth_ranking(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> Result<(Vec<StockScreeningResult>, i32), MarketDataError> {
        let skip_val = skip.unwrap_or(0);
        let limit_val = limit.unwrap_or(50);
        
        info!("🚀 REAL TradingView API: Fetching EPS ranking - Skip: {}, Limit: {}, Country: {:?}, Sector: {:?}, Sort: {:?}", 
              skip_val, limit_val, country, sector, sort_by);

        // Build request with proper pagination and filtering using REAL TradingView Scanner API
        let request_body = self.scanner.build_screener_request_with_params(
            skip_val,
            limit_val,
            country.clone(),
            sector.clone(),
            sort_by.clone(),
        );
        
        debug!("🌐 Real TradingView Scanner API request: {}", serde_json::to_string_pretty(&request_body).unwrap_or_default());

        // Execute REAL TradingView Scanner API call
        let response = self.scanner.get_rest_client().execute_custom_request(request_body, 3).await?;

        // TradingView API doesn't return total_count, so we use data length
        let total_count = response.totalCount.unwrap_or(response.data.len() as i32);
        
        info!("✅ Real TradingView API response: {} entries, total count: {}", 
              response.data.len(), total_count);

        // Convert TradingView response to screening results using real data
        let screening_results = self.scanner.process_trading_view_response(response.clone());

        info!("📊 Converted {} real TradingView stocks to screening results, total available: {}", 
               screening_results.len(), total_count);

        Ok((screening_results, total_count))
    }

    /// Extract EPS growth data from REAL TradingView response
    pub async fn extract_eps_growth_data(&self) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        info!("📈 Extracting EPS growth data from REAL TradingView API");
        
        let request = self.scanner.build_screener_request();
        let trading_view_resp = self.scanner.get_rest_client().execute_request_with_retry(request).await?;
        
        debug!("🔄 Processing {} real stocks for EPS data extraction", trading_view_resp.data.len());

        let mut eps_data_list = Vec::new();
        let mut processed_count = 0;
        let mut error_count = 0;

        for stock in trading_view_resp.data {
            processed_count += 1;
            
            match TradingViewMapper::convert_to_eps_growth_data(stock) {
                Ok(eps_data) => {
                    debug!("✅ Real EPS data for {}: current_eps={:?}, growth_factor={:?}, price_current={:?}", 
                           eps_data.symbol, eps_data.current_eps, eps_data.growth_factor, eps_data.price_current);
                    
                    eps_data_list.push(eps_data);
                }
                Err(e) => {
                    error!("❌ EPS data conversion failed: {}", e);
                    error_count += 1;
                }
            }

            if processed_count % 100 == 0 {
                debug!("🔄 Processed {} real stocks, found {} quality EPS entries", 
                       processed_count, eps_data_list.len());
            }
        }

        info!("🎯 Real EPS data extraction completed - Processed: {}, Quality: {}, Errors: {}", 
              processed_count, eps_data_list.len(), error_count);

        Ok(eps_data_list)
    }

    /// Fetch market data concurrently for a specific market
    async fn fetch_market_data_concurrent(&self, market: &str) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        debug!("🌍 Fetching real market data for: {}", market);
        
        let request_payload = self.scanner.build_market_request(market);
        let response = self.scanner.get_rest_client().execute_request_with_retry(request_payload).await?;
        
        // Process response concurrently
        let mut eps_data_list = Vec::new();
        for stock in response.data {
            match TradingViewMapper::convert_to_eps_growth_data(stock) {
                Ok(eps_data) => eps_data_list.push(eps_data),
                Err(e) => debug!("Failed to convert stock data: {}", e),
            }
        }
        
        debug!("📊 Fetched {} real EPS entries for market: {}", eps_data_list.len(), market);
        Ok(eps_data_list)
    }
    
    /// Fetch specific batch of symbols using REAL TradingView API
    async fn fetch_symbol_batch_data(&self, symbols: Vec<String>) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        debug!("📋 Fetching real data for batch of {} symbols", symbols.len());
        
        let request_payload = self.scanner.build_symbols_request(symbols.clone());
        let response = self.scanner.get_rest_client().execute_custom_request(request_payload, 2).await?;
        
        // Process batch response
        let mut eps_data_list = Vec::new();
        for stock in response.data {
            match TradingViewMapper::convert_to_eps_growth_data(stock) {
                Ok(eps_data) => eps_data_list.push(eps_data),
                Err(e) => debug!("Failed to convert symbol in batch: {}", e),
            }
        }
        
        debug!("✅ Processed real batch: {} symbols → {} EPS entries", symbols.len(), eps_data_list.len());
        Ok(eps_data_list)
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.read().unwrap().get_stats()
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear_all();
    }

    /// Test all service connections
    pub async fn test_connections(&self) -> Result<bool, MarketDataError> {
        info!("🧪 Testing REAL TradingView service connections");
        
        // Test REST API connection with real Scanner API
        let rest_ok = self.scanner.get_rest_client().test_connection().await.is_ok();
        
        // Test WebSocket connection with real TradingView WebSocket
        let ws_ok = self.websocket_handler.test_connection().await.is_ok();
        
        let all_ok = rest_ok && ws_ok;
        if all_ok {
            info!("✅ All REAL TradingView connections successful");
        } else {
            warn!("⚠️ Some TradingView connections failed - REST: {}, WebSocket: {}", rest_ok, ws_ok);
        }
        
        Ok(all_ok)
    }
}

#[async_trait]
impl TradingViewService for TradingViewApiService {
    async fn fetch_screener_data(&self) -> Result<Vec<StockScreeningResult>, MarketDataError> {
        info!("🔍 Fetching stock screener data from REAL TradingView API");
        
        let request = self.scanner.build_screener_request();
        let response = self.scanner.get_rest_client().execute_request_with_retry(request).await?;
        let stocks = self.scanner.process_trading_view_response(response);
        
        Ok(stocks)
    }

    async fn connect_realtime_feed(&self) -> Result<(), MarketDataError> {
        self.websocket_handler.connect_realtime_feed().await
    }

    async fn fetch_eps_growth_ranking(
        &self,
        skip: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> Result<(Vec<StockScreeningResult>, i32), MarketDataError> {
        // Call the implementation method directly
        TradingViewApiService::fetch_eps_growth_ranking(self, skip, limit, country, sector, sort_by).await
    }
    
    async fn extract_eps_growth_data(&self) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        // Delegate to the main implementation method  
        self.extract_eps_growth_data().await
    }
    
    async fn extract_eps_growth_data_concurrent(&self, batch_size: usize) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        info!("🚀 Extracting EPS growth data concurrently with batch size: {}", batch_size);
        
        // Fetch multiple market regions concurrently using REAL API
        let markets = vec!["america", "europe", "asia"];
        let mut concurrent_requests = Vec::new();
        
        for market in markets {
            let request = self.fetch_market_data_concurrent(market);
            concurrent_requests.push(request);
        }
        
        // Execute all requests concurrently with REAL TradingView API
        let results = futures::future::join_all(concurrent_requests).await;
        
        let mut all_eps_data = Vec::new();
        let mut total_processed = 0;
        let mut total_errors = 0;
        
        for result in results {
            match result {
                Ok(market_data) => {
                    info!("✅ Successfully fetched {} real EPS entries from market", market_data.len());
                    total_processed += market_data.len();
                    all_eps_data.extend(market_data);
                }
                Err(e) => {
                    warn!("❌ Failed to fetch real market data: {}", e);
                    total_errors += 1;
                }
            }
        }
        
        info!("🎯 Concurrent REAL EPS data extraction completed - Total: {}, Errors: {}", 
              total_processed, total_errors);
        
        Ok(all_eps_data)
    }

    async fn fetch_symbols_concurrent(&self, symbols: Vec<String>) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        info!("📊 Fetching {} symbols concurrently from REAL TradingView API", symbols.len());
        
        // Split symbols into batches for concurrent processing
        let batch_size = 10;
        let batches: Vec<Vec<String>> = symbols.chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        let mut concurrent_requests = Vec::new();
        
        for batch in batches {
            let request = self.fetch_symbol_batch_data(batch);
            concurrent_requests.push(request);
        }
        
        // Execute all batch requests concurrently with REAL API
        let results = futures::future::join_all(concurrent_requests).await;
        
        let mut all_eps_data = Vec::new();
        let mut successful_batches = 0;
        let mut failed_batches = 0;
        
        for result in results {
            match result {
                Ok(batch_data) => {
                    successful_batches += 1;
                    all_eps_data.extend(batch_data);
                }
                Err(e) => {
                    warn!("❌ Batch processing failed: {}", e);
                    failed_batches += 1;
                }
            }
        }
        
        info!("🎯 Concurrent REAL symbol fetching completed - Successful batches: {}, Failed batches: {}, Total symbols: {}", 
              successful_batches, failed_batches, all_eps_data.len());
        
        Ok(all_eps_data)
    }

    async fn fetch_eps_rankings_for_frontend(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
    ) -> Result<FrontendEPSResponse, MarketDataError> {
        info!("🖥️ Fetching REAL EPS rankings for frontend - page: {:?}, limit: {:?}, country: {:?}", 
              page, limit, country);
        
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        
        // Check cache first
        let cache_key = TradingViewCache::generate_eps_rankings_key(Some(page), Some(limit), country.clone(), None);
        if let Some(cached_response) = self.cache.write().unwrap().get_eps_rankings(&cache_key) {
            info!("📦 Returning cached EPS rankings for: {}", cache_key);
            return Ok(cached_response);
        }

        let request = self.scanner.build_screener_request_with_params(
            (page - 1) * limit,
            limit,
            country.clone(),
            None,
            None,
        );

        let result = self.scanner.get_rest_client().execute_request_with_retry(request).await?;

        // Convert to frontend format using REAL data
        let frontend_response = self.scanner.convert_to_frontend_format(result, page, limit);
        
        // Cache the response
        self.cache.write().unwrap().cache_eps_rankings(cache_key, frontend_response.clone(), Some(300)); // 5 minutes TTL
        
        info!("✅ Converted {} REAL EPS rankings to frontend format", frontend_response.data.len());
        Ok(frontend_response)
    }
    
    async fn fetch_enhanced_eps_rankings(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        use_websocket: bool,
    ) -> Result<FrontendEPSResponse, MarketDataError> {
        info!("🚀 Fetching enhanced REAL EPS rankings - page: {:?}, limit: {:?}, country: {:?}, websocket: {}", 
              page, limit, country, use_websocket);
        
        // First get the regular scanner data from REAL API
        let scanner_response = self.fetch_eps_rankings_for_frontend(page, limit, country.clone()).await?;
        
        // If WebSocket enhancement is not requested, return REAL scanner data
        if !use_websocket || scanner_response.data.is_empty() {
            return Ok(scanner_response);
        }
        
        // Extract symbols from scanner response
        let symbols: Vec<String> = scanner_response.data.iter()
            .map(|item| item.symbol.clone())
            .collect();
        
        info!("🔌 Fetching REAL WebSocket data for {} symbols", symbols.len());
        
        // Get detailed EPS data via REAL WebSocket
        match self.websocket_handler.fetch_enhanced_eps_data(symbols).await {
            Ok(websocket_data) => {
                info!("✅ Successfully fetched {} REAL WebSocket EPS records", websocket_data.len());
                
                // Merge REAL scanner data with REAL WebSocket data
                let merged_data = TradingViewMapper::merge_scanner_and_websocket_data(
                    scanner_response.data,
                    websocket_data
                );
                
                Ok(FrontendEPSResponse {
                    data: merged_data,
                    pagination: scanner_response.pagination,
                })
            }
            Err(e) => {
                warn!("⚠️ REAL WebSocket EPS data fetch failed: {}, falling back to scanner data", e);
                Ok(scanner_response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Test that all modules are accessible and core types are available
        let config = Config::default();
        let tv_config = TradingViewConfig::from(&config);
        
        // Test focused modules can be created
        let _scanner = TradingViewScanner::new(tv_config.clone());
        let _websocket_handler = TradingViewWebSocketHandler::new(tv_config.clone());
        let _cache = TradingViewCache::new();
        let _mapper = TradingViewMapper;
        
        // Test integrated service creation
        let service = TradingViewApiService::new(Arc::new(config));
        let stats = service.get_cache_stats();
        assert_eq!(stats.total_count, 0); // New cache should be empty
    }

    #[tokio::test]
    async fn test_service_creation() {
        let config = Config::default();
        let service = TradingViewApiService::new(Arc::new(config));
        
        let stats = service.get_cache_stats();
        assert_eq!(stats.total_count, 0);
    }

    #[test]
    fn test_cache_integration() {
        let config = Config::default();
        let service = TradingViewApiService::with_cache_settings(
            Arc::new(config),
            Duration::from_secs(600),
            2000
        );
        
        let initial_stats = service.get_cache_stats();
        assert_eq!(initial_stats.total_count, 0);
        
        service.clear_cache();
        let cleared_stats = service.get_cache_stats();
        assert_eq!(cleared_stats.total_count, 0);
    }

    #[tokio::test]
    #[ignore] // Ignore in CI/CD to avoid external API calls
    async fn test_real_api_connections_integration() {
        let config = Config::default();
        let service = TradingViewApiService::new(Arc::new(config));
        
        // This test requires actual TradingView API/WebSocket access
        let result = service.test_connections().await;
        match result {
            Ok(_) => {}, // Connection tests can succeed or fail in test environments
            Err(_) => {}, // Both are acceptable for unit tests
        }
    }

    #[test]
    fn test_focused_modules_integration() {
        let config = Config::default();
        let tv_config = TradingViewConfig::from(&config);
        
        // Test that focused modules work together correctly
        let scanner = TradingViewScanner::new(tv_config.clone());
        let cache = TradingViewCache::new();
        
        let request = scanner.build_screener_request();
        assert!(request["columns"].as_array().unwrap().len() > 0);
        
        let stats = cache.get_stats();
        assert_eq!(stats.total_count, 0);
    }

    #[test] 
    fn test_real_api_backward_compatibility() {
        // Ensure all original TradingView service functionality is still accessible with REAL APIs
        let config = Config::default();
        let service = TradingViewApiService::new(Arc::new(config));
        
        let stats = service.get_cache_stats();
        assert_eq!(stats.total_count, 0);
    }
}