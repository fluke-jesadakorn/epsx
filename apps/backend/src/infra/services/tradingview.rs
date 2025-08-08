// TradingView API integration service
use std::time::Duration;
use std::sync::Arc;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use async_trait::async_trait;

use crate::config::Config;
use crate::dom::entities::market_data::{
    TradingViewResponse, TradingViewStock, StockDataField,
    StockScreeningResult, QuoteSessionCreate, PhaseInfo, PhaseStatus, PhaseType,
    MarketDataError, FinancialFormatter
};
use crate::dom::entities::eps_growth::EPSGrowthData;
use super::websocket::WebSocketClient;
use super::tradingview_websocket::TradingViewWebSocketService;

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
            auth_token: config.auth.firebase_project_id.clone(), // Use appropriate auth token
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
    
    /// Fetch EPS growth ranking data
    async fn fetch_eps_growth_ranking(
        &self,
        limit: Option<i32>,
        skip: Option<i32>,
        sort_by: Option<String>,
    ) -> Result<Vec<StockScreeningResult>, MarketDataError>;
    
    /// Extract EPS growth data from TradingView response
    async fn extract_eps_growth_data(&self) -> Result<Vec<EPSGrowthData>, MarketDataError>;
    
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

/// TradingView API service implementation
pub struct TradingViewApiService {
    client: Client,
    ws_client: WebSocketClient,
    config: TradingViewConfig,
}

impl TradingViewApiService {
    /// Create new TradingView service
    pub fn new(app_config: Arc<Config>) -> Self {
        let config = TradingViewConfig::from(app_config.as_ref());
        
        let timeout_duration = Duration::from_secs(config.http_timeout_seconds);
        let client = ClientBuilder::new()
            .timeout(timeout_duration)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)")
            .build()
            .unwrap_or_else(|_| Client::new());

        let ws_client = WebSocketClient::new(
            &config.websocket_url,
            &config.auth_token
        );

        Self { 
            client,
            ws_client,
            config,
        }
    }

    /// Build request headers for TradingView API using exact format from capture
    fn get_request_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Exact headers from TradingView capture
        headers.insert("accept", reqwest::header::HeaderValue::from_static("application/json"));
        headers.insert("accept-language", reqwest::header::HeaderValue::from_static("th-TH,th;q=0.9,en;q=0.8"));
        headers.insert("cache-control", reqwest::header::HeaderValue::from_static("no-cache"));
        headers.insert("content-type", reqwest::header::HeaderValue::from_static("text/plain;charset=UTF-8"));
        headers.insert("pragma", reqwest::header::HeaderValue::from_static("no-cache"));
        headers.insert("priority", reqwest::header::HeaderValue::from_static("u=1, i"));
        
        // Security headers from capture
        headers.insert("sec-ch-ua", reqwest::header::HeaderValue::from_static(r#""Not;A=Brand";v="99", "Google Chrome";v="139", "Chromium";v="139""#));
        headers.insert("sec-ch-ua-mobile", reqwest::header::HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", reqwest::header::HeaderValue::from_static(r#""macOS""#));
        headers.insert("sec-fetch-dest", reqwest::header::HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", reqwest::header::HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", reqwest::header::HeaderValue::from_static("same-site"));
        
        // Referer from configuration or default
        let referer = self.config.referer_url.clone();
        if let Ok(referer_header) = reqwest::header::HeaderValue::from_str(&referer) {
            headers.insert("referer", referer_header);
        } else {
            headers.insert("referer", reqwest::header::HeaderValue::from_static("https://www.tradingview.com/"));
        }
        
        headers
    }

    /// Build screener request payload using exact format from TradingView capture
    fn build_screener_request(&self) -> serde_json::Value {
        json!({
            "columns": [
                "name", "description", "logoid", "update_mode", "type", "typespecs",
                "close", "pricescale", "minmov", "fractional", "minmove2", "currency",
                "change", "volume", "relative_volume_10d_calc", "market_cap_basic",
                "fundamental_currency_code", "price_earnings_ttm", "earnings_per_share_diluted_ttm",
                "earnings_per_share_diluted_yoy_growth_ttm", "dividends_yield_current",
                "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange"
            ],
            "filter": [
                {
                    "left": "earnings_per_share_diluted_yoy_growth_ttm",
                    "operation": "greater",
                    "right": 0
                },
                {
                    "left": "is_primary",
                    "operation": "equal",
                    "right": true
                }
            ],
            "ignore_unknown_fields": false,
            "options": { "lang": "en" },
            "price_conversion": { "to_currency": "usd" },
            "range": [0, 100],
            "sort": { "sortBy": "market_cap_basic", "sortOrder": "desc" },
            "symbols": {},
            "markets": [
                "america", "argentina", "australia", "austria", "bahrain", "bangladesh",
                "belgium", "brazil", "canada", "chile", "china", "colombia", "cyprus",
                "czech", "denmark", "egypt", "estonia", "finland", "france", "germany",
                "greece", "hongkong", "hungary", "iceland", "india", "indonesia",
                "ireland", "israel", "italy", "japan", "kenya", "kuwait", "latvia",
                "lithuania", "luxembourg", "malaysia", "mexico", "morocco", "netherlands",
                "newzealand", "nigeria", "norway", "pakistan", "peru", "philippines",
                "poland", "portugal", "qatar", "romania", "russia", "ksa", "serbia",
                "singapore", "slovakia", "rsa", "korea", "spain", "srilanka", "sweden",
                "switzerland", "taiwan", "thailand", "tunisia", "turkey", "uae", "uk",
                "venezuela", "vietnam"
            ],
            "filter2": {
                "operator": "and",
                "operands": [
                    {
                        "operation": {
                            "operator": "or",
                            "operands": [
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "stock"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has",
                                                    "right": ["common"]
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "stock"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has",
                                                    "right": ["preferred"]
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "dr"
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "fund"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has_none_of",
                                                    "right": ["etf"]
                                                }
                                            }
                                        ]
                                    }
                                }
                            ]
                        }
                    }
                ]
            }
        })
    }

    /// Convert TradingView stock data to EPS growth data
    fn convert_to_eps_growth_data(&self, stock: TradingViewStock) -> Result<EPSGrowthData, String> {
        debug!("Converting TradingView stock to EPS data: {}", stock.s);

        let get_number = |data: &[StockDataField], idx: usize| -> Option<f64> {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => Some(*n),
                Some(StockDataField::Integer(i)) => Some(*i as f64),
                _ => None
            }
        };

        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            data.get(idx)
                .map(|field| match field {
                    StockDataField::String(s) => s.clone(),
                    StockDataField::Number(n) => n.to_string(),
                    StockDataField::Integer(i) => i.to_string(),
                    StockDataField::Null => default.to_string(),
                })
                .unwrap_or_else(|| default.to_string())
        };

        // Extract symbol from full symbol (e.g., "NASDAQ:AAPL" -> "AAPL")
        let symbol = stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string();
        let name = get_string(&stock.d, 0, ""); // description
        let country = get_string(&stock.d, 12, "unknown"); // country
        let sector = get_string(&stock.d, 11, ""); // sector
        let exchange = get_string(&stock.d, 13, ""); // exchange
        
        // EPS data extraction with debug logging
        let current_eps = get_number(&stock.d, 10); // earnings_per_share_basic_ttm
        let qoq_growth = get_number(&stock.d, 15); // earnings_per_share_diluted_qoq_growth_fq
        let price_current = get_number(&stock.d, 2); // close
        let market_cap = get_number(&stock.d, 8).map(|mc| mc as i64); // market_cap_basic
        let volume = get_number(&stock.d, 6).map(|vol| vol as i64); // volume

        debug!("Extracted EPS data for {}: EPS={:?}, QoQ Growth={:?}, Price={:?}", 
               symbol, current_eps, qoq_growth, price_current);

        // Validate essential data
        if symbol.is_empty() {
            warn!("Empty symbol for TradingView stock: {:?}", stock.s);
            return Err("Empty symbol".to_string());
        }

        if country.is_empty() || country == "unknown" {
            warn!("Missing country for symbol: {}", symbol);
        }

        let eps_data = EPSGrowthData::new(
            symbol.clone(),
            name,
            country.to_lowercase(),
            sector,
            exchange,
            current_eps,
            qoq_growth,
            price_current,
            market_cap,
            volume,
        );

        // Validate the created data
        eps_data.validate().map_err(|e| {
            error!("EPS data validation failed for {}: {}", symbol, e);
            e
        })?;

        debug!("Successfully converted TradingView data to EPS data for: {}", symbol);
        Ok(eps_data)
    }

    /// Convert TradingView stock data to stock screening result
    fn convert_to_stock_screening_result(&self, stock: TradingViewStock) -> StockScreeningResult {
        let get_number = |data: &[StockDataField], idx: usize| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => 0.0
            }
        };

        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            data.get(idx)
                .map(|field| match field {
                    StockDataField::String(s) => s.clone(),
                    StockDataField::Number(n) => n.to_string(),
                    StockDataField::Integer(i) => i.to_string(),
                    StockDataField::Null => default.to_string(),
                })
                .unwrap_or_else(|| default.to_string())
        };

        let last = get_number(&stock.d, 17);
        let next = get_number(&stock.d, 18);
        let (entry_phase, phase_status) = if last != 0.0 && next != 0.0 {
            StockScreeningResult::get_analysis_phases(last as i64, next as i64)
        } else {
            (
                PhaseInfo {
                    date: "N/A".to_string(),
                    active: false,
                },
                PhaseStatus {
                    date: "N/A".to_string(),
                    phase_type: PhaseType::Monitor,
                    active: false,
                }
            )
        };

        StockScreeningResult {
            symbol: stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string(),
            name: get_string(&stock.d, 0, ""),
            value_index: StockScreeningResult::format_number(Some(get_number(&stock.d, 2))), // close
            growth_rate: StockScreeningResult::format_number(Some(get_number(&stock.d, 3))), // change
            activity_score: StockScreeningResult::format_large_number(get_number(&stock.d, 6)), // volume
            market_size: StockScreeningResult::format_large_number(get_number(&stock.d, 8)), // market_cap_basic
            growth_factor: StockScreeningResult::format_number(Some(get_number(&stock.d, 9))), // price_earnings_ttm
            sector: get_string(&stock.d, 11, "N/A"), // sector
            country: get_string(&stock.d, 12, "N/A"), // country
            exchange: get_string(&stock.d, 13, "N/A"), // exchange
            currency: get_string(&stock.d, 27, "N/A"), // currency
            metric_score: StockScreeningResult::format_number(Some(get_number(&stock.d, 10))), // earnings_per_share_basic_ttm
            growth_indicator: StockScreeningResult::format_number(Some(get_number(&stock.d, 15))), // earnings_per_share_diluted_qoq_growth_fq
            current_metric: StockScreeningResult::format_number(Some(get_number(&stock.d, 14))), // earnings_per_share_fq
            predicted_metric: StockScreeningResult::format_number(Some(get_number(&stock.d, 16))), // earnings_per_share_forecast_next_fq
            last_analysis_date: StockScreeningResult::format_date(if last != 0.0 { Some(last as i64) } else { None }), // earnings_release_date
            next_analysis_date: StockScreeningResult::format_date(if next != 0.0 { Some(next as i64) } else { None }), // earnings_release_next_date
            entry_phase,
            phase_status,
            start_buy: None,
            start_action: None,
            eps_growth: None,
            last_earnings_date: None,
        }
    }

    /// Process TradingView API response
    fn process_trading_view_response(&self, response: TradingViewResponse) -> Vec<StockScreeningResult> {
        response.data
            .into_iter()
            .map(|stock| self.convert_to_stock_screening_result(stock))
            .collect()
    }

    /// Convert TradingView response to frontend format
    fn convert_to_frontend_format(&self, response: TradingViewResponse, page: i32, limit: i32) -> FrontendEPSResponse {
        let frontend_data: Vec<FrontendEPSData> = response.data
            .into_iter()
            .map(|stock| self.map_to_frontend_eps_data(stock))
            .collect();

        let total = response.total_count;
        let total_pages = (total as f64 / limit as f64).ceil() as i32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        FrontendEPSResponse {
            data: frontend_data,
            pagination: FrontendPagination {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }

    /// Map TradingView stock to frontend EPS data format
    fn map_to_frontend_eps_data(&self, stock: TradingViewStock) -> FrontendEPSData {
        // Helper functions to extract data from the array
        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            match data.get(idx) {
                Some(StockDataField::String(s)) => s.clone(),
                Some(StockDataField::Number(n)) => n.to_string(),
                Some(StockDataField::Integer(i)) => i.to_string(),
                _ => default.to_string(),
            }
        };

        let get_number = |data: &[StockDataField], idx: usize, default: f64| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => default,
            }
        };

        // Extract data according to TradingView column order from your capture:
        // [name, description, logoid, update_mode, type, typespecs, close, pricescale, minmov, fractional, 
        //  minmove2, currency, change, volume, relative_volume_10d_calc, market_cap_basic, fundamental_currency_code, 
        //  price_earnings_ttm, earnings_per_share_diluted_ttm, earnings_per_share_diluted_yoy_growth_ttm, 
        //  dividends_yield_current, sector.tr, market, sector, AnalystRating, AnalystRating.tr, exchange]
        
        let symbol = stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string(); // Extract symbol from "NASDAQ:NVDA"
        let company_name = get_string(&stock.d, 1, "Unknown Company"); // description
        let price_current = get_number(&stock.d, 6, 0.0); // close
        let volume = get_number(&stock.d, 13, 0.0) as i64; // volume  
        let market_cap = get_number(&stock.d, 15, 0.0) as i64; // market_cap_basic
        let current_eps = get_number(&stock.d, 18, 0.0); // earnings_per_share_diluted_ttm
        let qoq_growth = get_number(&stock.d, 19, 0.0); // earnings_per_share_diluted_yoy_growth_ttm
        let sector = get_string(&stock.d, 21, "Unknown"); // sector.tr
        let country = get_string(&stock.d, 22, "unknown"); // market

        // Calculate ranking score based on EPS, growth, and market cap
        let ranking_score = self.calculate_ranking_score(current_eps, qoq_growth, market_cap as f64, price_current);

        FrontendEPSData {
            id: Uuid::new_v4().to_string(),
            symbol,
            company_name,
            current_eps,
            qoq_growth,
            market_cap,
            price_current,
            volume,
            country,
            sector,
            ranking_score,
        }
    }

    /// Calculate ranking score based on multiple factors
    fn calculate_ranking_score(&self, current_eps: f64, qoq_growth: f64, market_cap: f64, price: f64) -> f64 {
        // Weighted scoring algorithm
        let eps_weight = 0.3;
        let growth_weight = 0.4;
        let market_cap_weight = 0.2;
        let price_weight = 0.1;

        // Normalize values (simple approach)
        let eps_score = (current_eps * 10.0).min(100.0).max(0.0);
        let growth_score = (qoq_growth / 100.0 * 100.0).min(100.0).max(0.0);
        let market_cap_score = (market_cap / 1_000_000_000_000.0 * 100.0).min(100.0).max(0.0);
        let price_score = (price / 1000.0 * 100.0).min(100.0).max(0.0);

        (eps_score * eps_weight + growth_score * growth_weight + 
         market_cap_score * market_cap_weight + price_score * price_weight).round()
    }
}

#[async_trait]
impl TradingViewService for TradingViewApiService {
    async fn fetch_screener_data(&self) -> Result<Vec<StockScreeningResult>, MarketDataError> {
        info!("Fetching stock screener data from TradingView");
        
        // Setup retry strategy with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        let result = Retry::spawn(retry_strategy, || async {
            info!("Making request to TradingView API");
            debug!("Attempting to fetch stock screener data from TradingView");

            let response = self
                .client
                .post(&self.config.scanner_api_url)
                .headers(self.get_request_headers())
                .json(&self.build_screener_request())
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to fetch from TradingView: {}", e);
                    MarketDataError::NetworkError(e.to_string())
                })?;

            if !response.status().is_success() {
                let error_msg = format!("TradingView API returned status code: {}", response.status());
                error!("{}", error_msg);
                return Err(MarketDataError::ExternalApiError(error_msg));
            }

            let trading_view_resp: TradingViewResponse = response.json().await
                .map_err(|e| {
                    error!("Failed to parse TradingView response: {}", e);
                    MarketDataError::NetworkError(e.to_string())
                })?;

            debug!("Successfully fetched and parsed stock screener data");
            Ok(trading_view_resp)
        }).await?;

        let stocks = self.process_trading_view_response(result);
        
        Ok(stocks)
    }

    async fn connect_realtime_feed(&self) -> Result<(), MarketDataError> {
        // Generate a unique session ID
        let session_id = format!("qs_{}", Uuid::new_v4().to_string().replace("-", "").chars().take(10).collect::<String>());
        
        // Connect to WebSocket
        let connection = self.ws_client.connect::<serde_json::Value>().await?;
        
        // Create quote session message
        let quote_session = QuoteSessionCreate {
            session_id: session_id.clone(),
            symbols: vec![], // Initialize with empty symbols for now
        };
        
        // Send the message
        connection.send_message(&quote_session).await.map_err(|e| MarketDataError::NetworkError(e.to_string()))?;
        
        // Start receiving messages
        let mut receiver = connection.start_receive_loop().await.map_err(|e| MarketDataError::NetworkError(e.to_string()))?;
        
        // Process received messages in a separate task
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                info!("Received screener message: {:?}", message);
            }
        });

        Ok(())
    }

    async fn fetch_eps_growth_ranking(
        &self,
        limit: Option<i32>,
        skip: Option<i32>,
        sort_by: Option<String>,
    ) -> Result<Vec<StockScreeningResult>, MarketDataError> {
        // For now, use the same screener data and apply sorting/filtering
        // In a real implementation, this would use a different API endpoint
        let mut data = self.fetch_screener_data().await?;

        // Apply sorting based on field name
        if let Some(sort_field) = sort_by {
            data.sort_by(|a, b| {
                match sort_field.as_str() {
                    "symbol" => a.symbol.cmp(&b.symbol),
                    "name" => a.name.cmp(&b.name),
                    "sector" => a.sector.cmp(&b.sector),
                    "country" => a.country.cmp(&b.country),
                    "exchange" => a.exchange.cmp(&b.exchange),
                    // For numeric fields, parse and compare
                    "value_index" => {
                        let a_val: f64 = a.value_index.parse().unwrap_or(0.0);
                        let b_val: f64 = b.value_index.parse().unwrap_or(0.0);
                        b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    "growth_rate" => {
                        let a_val: f64 = a.growth_rate.parse().unwrap_or(0.0);
                        let b_val: f64 = b.growth_rate.parse().unwrap_or(0.0);
                        b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
                    },
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }

        // Apply pagination
        let skip_count = skip.unwrap_or(0) as usize;
        let limit_count = limit.map(|l| l as usize);

        let filtered_data: Vec<_> = data
            .into_iter()
            .skip(skip_count)
            .take(limit_count.unwrap_or(usize::MAX))
            .collect();

        Ok(filtered_data)
    }

    async fn extract_eps_growth_data(&self) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        info!("Extracting EPS growth data from TradingView");
        
        // Fetch raw screener data first
        let response = self.fetch_screener_data().await?;
        debug!("Processing {} stocks for EPS data extraction", response.len());

        let mut eps_data_list = Vec::new();
        let mut processed_count = 0;
        let mut filtered_count = 0;
        let mut error_count = 0;

        for stock_screening in response {
            processed_count += 1;
            
            // Convert StockScreeningResult back to TradingViewStock-like data
            // Note: This is a simplified approach, in production you'd want to 
            // extract EPS data directly from the TradingView response before conversion
            
            // For now, we'll create a minimal EPS data structure from the screening result
            let eps_data = EPSGrowthData::new(
                stock_screening.symbol.clone(),
                stock_screening.name.clone(),
                stock_screening.country.to_lowercase(),
                stock_screening.sector.clone(),
                stock_screening.exchange.clone(),
                stock_screening.current_metric.parse().ok(),
                stock_screening.growth_indicator.parse().ok(),
                stock_screening.value_index.parse().ok(),
                None, // market_cap - would need to be extracted from screening result
                None, // volume - would need to be extracted from screening result
            );

            // Validate and filter quality data
            match eps_data.validate() {
                Ok(()) => {
                    if eps_data.has_quality_data() {
                        debug!("Adding quality EPS data for: {}", eps_data.symbol);
                        eps_data_list.push(eps_data);
                    } else {
                        debug!("Filtering out {} due to incomplete data", eps_data.symbol);
                        filtered_count += 1;
                    }
                }
                Err(e) => {
                    warn!("Validation error for {}: {}", eps_data.symbol, e);
                    error_count += 1;
                }
            }

            if processed_count % 100 == 0 {
                debug!("Processed {} stocks, found {} quality EPS entries", 
                       processed_count, eps_data_list.len());
            }
        }

        info!("EPS data extraction completed - Processed: {}, Quality: {}, Filtered: {}, Errors: {}", 
              processed_count, eps_data_list.len(), filtered_count, error_count);

        Ok(eps_data_list)
    }

    async fn fetch_eps_rankings_for_frontend(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
    ) -> Result<FrontendEPSResponse, MarketDataError> {
        info!("Fetching EPS rankings for frontend - page: {:?}, limit: {:?}, country: {:?}", 
              page, limit, country);
        
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        
        // Setup retry strategy with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        let result = Retry::spawn(retry_strategy, || async {
            info!("Making request to TradingView scanner API");
            debug!("Attempting to fetch EPS rankings from TradingView");

            // Build custom request payload with pagination and filtering
            let mut request_payload = self.build_screener_request();
            
            // Calculate range for pagination
            let start_range = (page - 1) * limit;
            let end_range = start_range + limit;
            request_payload["range"] = json!([start_range, end_range]);
            
            // Add country filtering if specified
            if let Some(ref country_filter) = country {
                if !country_filter.is_empty() && country_filter != "all" {
                    let markets = if country_filter == "america" {
                        vec!["america"]
                    } else {
                        vec![country_filter.as_str()]
                    };
                    request_payload["markets"] = json!(markets);
                }
            }

            let response = self
                .client
                .post(&self.config.scanner_api_url)
                .headers(self.get_request_headers())
                .json(&request_payload)
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to fetch from TradingView scanner: {}", e);
                    MarketDataError::NetworkError(e.to_string())
                })?;

            if !response.status().is_success() {
                let error_msg = format!("TradingView scanner API returned status code: {}", response.status());
                error!("{}", error_msg);
                return Err(MarketDataError::ExternalApiError(error_msg));
            }

            let trading_view_resp: TradingViewResponse = response.json().await
                .map_err(|e| {
                    error!("Failed to parse TradingView scanner response: {}", e);
                    MarketDataError::NetworkError(e.to_string())
                })?;

            debug!("Successfully fetched and parsed EPS rankings from scanner");
            Ok(trading_view_resp)
        }).await?;

        // Convert to frontend format
        let frontend_response = self.convert_to_frontend_format(result, page, limit);
        
        info!("Converted {} EPS rankings to frontend format", frontend_response.data.len());
        Ok(frontend_response)
    }
    
    async fn fetch_enhanced_eps_rankings(
        &self,
        page: Option<i32>,
        limit: Option<i32>,
        country: Option<String>,
        use_websocket: bool,
    ) -> Result<FrontendEPSResponse, MarketDataError> {
        info!("Fetching enhanced EPS rankings - page: {:?}, limit: {:?}, country: {:?}, websocket: {}", 
              page, limit, country, use_websocket);
        
        // First get the regular scanner data
        let scanner_response = self.fetch_eps_rankings_for_frontend(page, limit, country.clone()).await?;
        
        // If WebSocket enhancement is not requested, return scanner data
        if !use_websocket || scanner_response.data.is_empty() {
            return Ok(scanner_response);
        }
        
        // Extract symbols from scanner response
        let symbols: Vec<String> = scanner_response.data.iter()
            .map(|item| item.symbol.clone())
            .collect();
        
        info!("Fetching WebSocket data for {} symbols", symbols.len());
        
        // Get detailed EPS data via WebSocket
        let mut websocket_service = TradingViewWebSocketService::new();
        match websocket_service.connect_and_fetch_eps_data(symbols).await {
            Ok(websocket_data) => {
                info!("Successfully fetched {} WebSocket EPS records", websocket_data.len());
                
                // Convert WebSocket data to frontend format
                let enhanced_frontend_data = websocket_service.convert_to_frontend_format(websocket_data);
                
                // Merge scanner data with WebSocket data
                let merged_data = self.merge_scanner_and_websocket_data(
                    scanner_response.data,
                    enhanced_frontend_data
                );
                
                Ok(FrontendEPSResponse {
                    data: merged_data,
                    pagination: scanner_response.pagination,
                })
            }
            Err(e) => {
                warn!("WebSocket EPS data fetch failed: {}, falling back to scanner data", e);
                Ok(scanner_response)
            }
        }
    }
    
}

impl TradingViewApiService {
    /// Merge scanner data with WebSocket data, preferring WebSocket details
    fn merge_scanner_and_websocket_data(
        &self,
        scanner_data: Vec<FrontendEPSData>,
        websocket_data: Vec<FrontendEPSData>
    ) -> Vec<FrontendEPSData> {
        use std::collections::HashMap;
        
        // Create a map of WebSocket data by symbol for fast lookup
        let websocket_map: HashMap<String, FrontendEPSData> = websocket_data
            .into_iter()
            .map(|item| (item.symbol.clone(), item))
            .collect();
        
        // Merge data, preferring WebSocket data when available
        scanner_data.into_iter().map(|mut scanner_item| {
            if let Some(websocket_item) = websocket_map.get(&scanner_item.symbol) {
                // Use WebSocket data for EPS details
                scanner_item.current_eps = websocket_item.current_eps;
                scanner_item.qoq_growth = websocket_item.qoq_growth;
                scanner_item.ranking_score = websocket_item.ranking_score;
                
                // Keep other data from scanner (company name, market cap, etc.)
                // as it may be more accurate from the scanner API
                debug!("Enhanced {} with WebSocket EPS data", scanner_item.symbol);
            }
            scanner_item
        }).collect()
    }
}