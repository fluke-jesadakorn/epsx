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
use chrono::Utc;

use crate::config::Config;
use crate::infra::cache::{Cache, CacheExt};
use crate::stock::common::{StockServiceError, TradingViewResponse, TradingViewStock, StockDataField, NumberFormatter, WebSocketClient};
use super::models::{TableDataMetrics, QuoteSessionCreate};

pub struct ScreenerService {
    client: Client,
    ws_client: WebSocketClient,
    config: Arc<Config>,
    cache: Arc<dyn Cache>,
}

impl ScreenerService {
    pub fn new(config: Arc<Config>, cache: Arc<dyn Cache>) -> Self {
        let timeout_duration = Duration::from_secs(config.external_services.tradingview.http_timeout_seconds);
        let client = ClientBuilder::new()
            .timeout(timeout_duration)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)")
            .build()
            .unwrap_or_else(|_| Client::new());

        let ws_client = WebSocketClient::new(
            "wss://data.tradingview.com/socket.io/websocket?from=screener",
            &config.auth.firebase_project_id // Use appropriate auth token from config
        );

        Self { 
            client,
            ws_client,
            config,
            cache,
        }
    }

    // WebSocket-specific methods
    pub async fn connect_screener(&self) -> Result<(), StockServiceError> {
        // Generate a unique session ID
        let session_id = format!("qs_{}", Uuid::new_v4().to_string().replace("-", "").chars().take(10).collect::<String>());
        
        // Connect to WebSocket
        let connection = self.ws_client.connect::<serde_json::Value>().await?;
        
        // Create quote session message
        let quote_session = QuoteSessionCreate::new(&session_id);
        
        // Send the message
        connection.send_message(&quote_session).await?;
        
        // Start receiving messages
        let mut receiver = connection.start_receive_loop().await?;
        
        // Process received messages in a separate task
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                info!("Received screener message: {:?}", message);
            }
        });

        Ok(())
    }

    async fn get_todays_data(&self) -> Result<Option<Vec<TableDataMetrics>>, StockServiceError> {
        let today_key = format!("stock:screener:data:{}", chrono::Utc::now().format("%Y-%m-%d"));
        
        match self.cache.get::<Vec<TableDataMetrics>>(&today_key).await {
            Ok(data) => {
                if data.is_some() {
                    debug!("Retrieved today's stock data from cache");
                } else {
                    debug!("No cached data found for today");
                }
                Ok(data)
            }
            Err(e) => {
                debug!("Cache error when retrieving today's data: {}", e);
                Ok(None) // Continue without cache on error
            }
        }
    }

    async fn save_stock_data(&self, stocks: Vec<TableDataMetrics>) -> Result<(), StockServiceError> {
        let today_key = format!("stock:screener:data:{}", Utc::now().format("%Y-%m-%d"));
        let cache_ttl = 300; // 5 minutes for real-time data
        
        match self.cache.set(&today_key, &stocks, Some(cache_ttl)).await {
            Ok(_) => {
                debug!("Successfully cached {} stock records with TTL {}s", stocks.len(), cache_ttl);
            }
            Err(e) => {
                // Don't fail the operation if caching fails
                debug!("Failed to cache stock data (operation continues): {}", e);
            }
        }
        
        Ok(())
    }

    /// Cache individual stock data by symbol
    async fn cache_stock_by_symbol(&self, stock: &TableDataMetrics) -> Result<(), StockServiceError> {
        let symbol_key = format!("stock:screener:symbol:{}", stock.symbol);
        let cache_ttl = 300; // 5 minutes for real-time data

        match self.cache.set(&symbol_key, stock, Some(cache_ttl)).await {
            Ok(_) => {
                debug!("Cached stock data for symbol: {}", stock.symbol);
            }
            Err(e) => {
                debug!("Failed to cache stock data for symbol {} (operation continues): {}", stock.symbol, e);
            }
        }

        Ok(())
    }

    /// Get cached stock data by symbol
    async fn get_cached_stock_by_symbol(&self, symbol: &str) -> Option<TableDataMetrics> {
        let symbol_key = format!("stock:screener:symbol:{}", symbol);
        
        match self.cache.get::<TableDataMetrics>(&symbol_key).await {
            Ok(data) => {
                if data.is_some() {
                    debug!("Retrieved cached data for symbol: {}", symbol);
                } else {
                    debug!("No cached data found for symbol: {}", symbol);
                }
                data
            }
            Err(e) => {
                debug!("Cache error when retrieving data for symbol {}: {}", symbol, e);
                None
            }
        }
    }

    fn get_request_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("accept", reqwest::header::HeaderValue::from_static("text/plain, */*; q=0.01"));
        headers.insert("accept-language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", reqwest::header::HeaderValue::from_static("application/json"));
        
        // Use configuration for origin and referer headers
        if let Ok(origin) = reqwest::header::HeaderValue::from_str(&self.config.external_services.tradingview.origin_url) {
            headers.insert("origin", origin);
        }
        if let Ok(referer) = reqwest::header::HeaderValue::from_str(&self.config.external_services.tradingview.referer_url) {
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

    pub async fn fetch_stock_screener_data(&self) -> Result<Vec<TableDataMetrics>, StockServiceError> {
        // First try to get today's data from cache
        if let Some(data) = self.get_todays_data().await? {
            info!("Returning today's cached data");
            return Ok(data);
        }

        // If no data for today, fetch from TradingView
        info!("No data for today, fetching from TradingView");
        
        // Setup retry strategy with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        let result = Retry::spawn(retry_strategy, || async {
            info!("Making request to TradingView API");
            debug!("Attempting to fetch stock screener data from TradingView");

            let response = self
                .client
                .post(&self.config.external_services.tradingview.scanner_api_url)
                .headers(self.get_request_headers())
                .json(&self.build_screener_request())
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to fetch from TradingView: {}", e);
                    StockServiceError::NetworkError(e)
                })?;

            if !response.status().is_success() {
                let error_msg = format!("TradingView API returned status code: {}", response.status());
                error!("{}", error_msg);
                return Err(StockServiceError::TradingViewError(error_msg));
            }

            let trading_view_resp: TradingViewResponse = response.json().await
                .map_err(|e| {
                    error!("Failed to parse TradingView response: {}", e);
                    StockServiceError::NetworkError(e)
                })?;

            debug!("Successfully fetched and parsed stock screener data");
            Ok(trading_view_resp)
        }).await?;

        let stocks = self.process_trading_view_response(result);

        // Save the fetched data to cache (both daily data and individual symbols)
        self.save_stock_data(stocks.clone()).await?;
        
        // Cache individual stock data by symbol for faster lookups
        for stock in &stocks {
            let _ = self.cache_stock_by_symbol(stock).await; // Don't fail if individual caching fails
        }
        
        Ok(stocks)
    }

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

    fn process_trading_view_response(&self, response: TradingViewResponse) -> Vec<TableDataMetrics> {
        response.data
            .into_iter()
            .map(|stock| self.convert_to_table_metrics(stock))
            .collect()
    }

    fn convert_to_table_metrics(&self, stock: TradingViewStock) -> TableDataMetrics {
        let get_number = |data: &[StockDataField], idx: usize| -> f64 {
            match data.get(idx) {
                Some(&StockDataField::Number(n)) => n,
                _ => 0.0
            }
        };

        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            data.get(idx)
                .map(|field| match field {
                    StockDataField::String(s) => s.clone(),
                    StockDataField::Number(n) => n.to_string(),
                })
                .unwrap_or_else(|| default.to_string())
        };

        let last = get_number(&stock.data, 17);
        let next = get_number(&stock.data, 18);
        let (entry_phase, phase_status) = if last != 0.0 && next != 0.0 {
            TableDataMetrics::get_analysis_phases(last as i64, next as i64)
        } else {
            (
                crate::stock::common::PhaseInfo {
                    date: "N/A".to_string(),
                    active: false,
                },
                crate::stock::common::PhaseStatus {
                    date: "N/A".to_string(),
                    phase_type: crate::stock::common::PhaseType::Monitor,
                    active: false,
                }
            )
        };

        TableDataMetrics {
            symbol: stock.symbol.split(':').nth(1).unwrap_or(&stock.symbol).to_string(),
            name: get_string(&stock.data, 0, ""),
            value_index: TableDataMetrics::format_number(Some(get_number(&stock.data, 2))), // close
            growth_rate: TableDataMetrics::format_number(Some(get_number(&stock.data, 3))), // change
            activity_score: TableDataMetrics::format_large_number(get_number(&stock.data, 6)), // volume
            market_size: TableDataMetrics::format_large_number(get_number(&stock.data, 8)), // market_cap_basic
            growth_factor: TableDataMetrics::format_number(Some(get_number(&stock.data, 9))), // price_earnings_ttm
            sector: get_string(&stock.data, 11, "N/A"), // sector
            country: get_string(&stock.data, 12, "N/A"), // country
            exchange: get_string(&stock.data, 13, "N/A"), // exchange
            currency: get_string(&stock.data, 27, "N/A"), // currency
            metric_score: TableDataMetrics::format_number(Some(get_number(&stock.data, 10))), // earnings_per_share_basic_ttm
            growth_indicator: TableDataMetrics::format_number(Some(get_number(&stock.data, 15))), // earnings_per_share_diluted_qoq_growth_fq
            current_metric: TableDataMetrics::format_number(Some(get_number(&stock.data, 14))), // earnings_per_share_fq
            predicted_metric: TableDataMetrics::format_number(Some(get_number(&stock.data, 16))), // earnings_per_share_forecast_next_fq
            last_analysis_date: TableDataMetrics::format_date(if last != 0.0 { Some(last as i64) } else { None }), // earnings_release_date
            next_analysis_date: TableDataMetrics::format_date(if next != 0.0 { Some(next as i64) } else { None }), // earnings_release_next_date
            entry_phase,
            phase_status,
            start_buy: None,
            start_action: None,
            eps_growth: None,
            last_earnings_date: None,
        }
    }

    // REMOVED: fetch_eps_growth_ranking - was not a real algorithm, just sorting existing data
    // This was a placeholder implementation that should be replaced with actual EPS analysis
    // when proper market data integration is implemented
}
