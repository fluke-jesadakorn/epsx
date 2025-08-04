// TradingView API integration service
use std::time::Duration;
use std::sync::Arc;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{debug, error, info};
use uuid::Uuid;
use async_trait::async_trait;

use crate::config::Config;
use crate::dom::entities::market_data::{
    TradingViewResponse, TradingViewStock, StockDataField,
    StockScreeningResult, QuoteSessionCreate, PhaseInfo, PhaseStatus, PhaseType,
    MarketDataError, FinancialFormatter
};
use super::websocket::WebSocketClient;

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
            scanner_api_url: config.external_services.tradingview.scanner_api_url.clone(),
            websocket_url: "wss://data.tradingview.com/socket.io/websocket?from=screener".to_string(),
            origin_url: config.external_services.tradingview.origin_url.clone(),
            referer_url: config.external_services.tradingview.referer_url.clone(),
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

    /// Build request headers for TradingView API
    fn get_request_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("accept", reqwest::header::HeaderValue::from_static("text/plain, */*; q=0.01"));
        headers.insert("accept-language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", reqwest::header::HeaderValue::from_static("application/json"));
        
        // Use configuration for origin and referer headers
        if let Ok(origin) = reqwest::header::HeaderValue::from_str(&self.config.origin_url) {
            headers.insert("origin", origin);
        }
        if let Ok(referer) = reqwest::header::HeaderValue::from_str(&self.config.referer_url) {
            headers.insert("referer", referer);
        }
        
        headers.insert("sec-ch-ua", reqwest::header::HeaderValue::from_static(r#""Not A(Brand";v="99", "Google Chrome";v="121""#));
        headers.insert("sec-ch-ua-mobile", reqwest::header::HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", reqwest::header::HeaderValue::from_static(r#""macOS""#));
        headers.insert("sec-fetch-dest", reqwest::header::HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", reqwest::header::HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", reqwest::header::HeaderValue::from_static("same-site"));
        headers
    }

    /// Build screener request payload
    fn build_screener_request(&self) -> serde_json::Value {
        json!({
            "filter": [
                { "left": "High.All", "operation": "eless", "right": "high" },
                { "left": "is_primary", "operation": "equal", "right": true },
                { "left": "active_symbol", "operation": "equal", "right": true },
                { "left": "basic_eps_net_income", "operation": "greater", "right": 0 },
                { "left": "earnings_per_share_diluted_ttm", "operation": "greater", "right": 0 },
                { "left": "last_annual_eps", "operation": "greater", "right": 0 },
                { "left": "earnings_per_share_forecast_next_fq", "operation": "greater", "right": 0 },
                { "left": "earnings_per_share_fq", "operation": "greater", "right": 0 },
                { "left": "earnings_per_share_diluted_qoq_growth_fq", "operation": "greater", "right": 0 }
            ],
            "options": { "lang": "en" },
            "markets": [
                "america", "uk", "india", "spain", "russia", "australia", "brazil",
                "japan", "newzealand", "turkey", "switzerland", "hongkong", "taiwan",
                "netherlands", "belgium", "portugal", "france", "moxico", "canada",
                "colombia", "uae", "nigeria", "singapore", "germany", "pakistan",
                "peru", "poland", "italy", "argentina", "israel", "ireland", "egypt",
                "srilanka", "serbia", "chile", "china", "malaysia", "morocco", "ksa",
                "bahrain", "qatar", "indonesia", "finland", "iceland", "denmark",
                "romania", "hungary", "sweden", "slovakia", "lithuania", "luxembourg",
                "estonia", "latvia", "vietnam", "rsa", "thailand", "tunisia", "korea",
                "kenya", "kuwait", "norway", "philippines", "greece", "venezuela",
                "cyprus", "bangladesh", "austria", "czech"
            ],
            "symbols": { "query": { "types": [] }, "tickers": [] },
            "columns": [
                "logoid", "name", "close", "change", "change_abs", "Recommend.All",
                "volume", "Value.Traded", "market_cap_basic", "price_earnings_ttm",
                "earnings_per_share_basic_ttm", "sector", "country", "exchange",
                "earnings_per_share_fq", "earnings_per_share_diluted_qoq_growth_fq",
                "earnings_per_share_forecast_next_fq", "earnings_release_date",
                "earnings_release_next_date", "description", "type", "subtype",
                "update_mode", "pricescale", "minmov", "fractional", "minmove2",
                "currency", "fundamental_currency_code"
            ],
            "sort": { "sortBy": "volume", "sortOrder": "desc" },
            "range": [0, 1000]
        })
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
}