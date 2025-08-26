use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use crate::config::env::get_env_var;

use crate::app::ports::services::{StockDataSvc, StockServiceError, StockPrice, MarketStatus, SymbolInfo};
use crate::dom::values::Symbol;

/// Configuration for market data providers
#[derive(Debug, Clone)]
pub struct MarketDataConfig {
    pub api_key: String,
    pub base_url: String,
    pub rate_limit: u32, // requests per minute
    pub timeout_seconds: u64,
}

/// Alpha Vantage market data provider
pub struct AlphaVantageService {
    client: Client,
    config: MarketDataConfig,
    cache: std::sync::Arc<std::sync::Mutex<HashMap<String, CachedPrice>>>,
}

#[derive(Debug, Clone)]
struct CachedPrice {
    price: StockPrice,
    cached_at: chrono::DateTime<chrono::Utc>,
    ttl_seconds: u64,
}

impl CachedPrice {
    fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.num_seconds() > self.ttl_seconds as i64
    }
}

#[derive(Debug, Deserialize)]
struct AlphaVantageQuote {
    #[serde(rename = "Global Quote")]
    global_quote: GlobalQuote,
}

#[derive(Debug, Deserialize)]  
struct GlobalQuote {
    #[serde(rename = "01. symbol")]
    #[allow(dead_code)]
    symbol: String,
    #[serde(rename = "05. price")]
    price: String,
    #[serde(rename = "06. volume")]
    volume: String,
    #[serde(rename = "09. change")]
    change: String,
    #[serde(rename = "10. change percent")]
    change_percent: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageTimeSeries {
    #[serde(rename = "Meta Data")]
    #[allow(dead_code)]
    metadata: TimeSeriesMetadata,
    #[serde(rename = "Time Series (Daily)")]
    time_series: Option<HashMap<String, DailyData>>,
}

#[derive(Debug, Deserialize)]
struct TimeSeriesMetadata {
    #[serde(rename = "2. Symbol")]
    #[allow(dead_code)]
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    #[serde(rename = "1. open")]
    #[allow(dead_code)]
    open: String,
    #[serde(rename = "2. high")]
    #[allow(dead_code)]
    high: String,
    #[serde(rename = "3. low")]
    #[allow(dead_code)]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

impl AlphaVantageService {
    pub fn new(config: MarketDataConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            config,
            cache: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = get_env_var("ALPHA_VANTAGE_API_KEY")
            .map_err(|_| "ALPHA_VANTAGE_API_KEY environment variable not set")?;

        let config = MarketDataConfig {
            api_key,
            base_url: "https://www.alphavantage.co/query".to_string(),
            rate_limit: 5, // Alpha Vantage free tier: 5 calls per minute
            timeout_seconds: 30,
        };

        Ok(Self::new(config))
    }

    fn get_cached_price(&self, symbol: &Symbol) -> Option<StockPrice> {
        let cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.get(&symbol.to_string()) {
            if !cached.is_expired() {
                return Some(cached.price.clone());
            }
        }
        None
    }

    fn cache_price(&self, symbol: &Symbol, price: StockPrice, ttl_seconds: u64) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(symbol.to_string(), CachedPrice {
            price,
            cached_at: chrono::Utc::now(),
            ttl_seconds,
        });
    }

    async fn make_request(&self, function: &str, params: &[(&str, &str)]) -> Result<serde_json::Value, StockServiceError> {
        let mut url = format!("{}?function={}&apikey={}", 
                             self.config.base_url, function, self.config.api_key);
        
        for (key, value) in params {
            url.push_str(&format!("&{}={}", key, value));
        }

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| StockServiceError::ExternalError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(StockServiceError::ExternalError(
                format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown"))
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| StockServiceError::ExternalError(e.to_string()))?;

        // Check for API error responses
        if let Some(error_message) = json.get("Error Message") {
            return Err(StockServiceError::ExternalError(error_message.to_string()));
        }

        if let Some(_note) = json.get("Note") {
            return Err(StockServiceError::RateLimitExceeded);
        }

        Ok(json)
    }
}

#[async_trait]
impl StockDataSvc for AlphaVantageService {
    async fn get_real_time_price(&self, symbol: &Symbol) -> Result<StockPrice, StockServiceError> {
        // Check cache first (1 minute TTL for real-time prices)
        if let Some(cached_price) = self.get_cached_price(symbol) {
            return Ok(cached_price);
        }

        let symbol_str = symbol.to_string();
        let params = [("symbol", symbol_str.as_str())];
        let json = self.make_request("GLOBAL_QUOTE", &params).await?;

        let quote: AlphaVantageQuote = serde_json::from_value(json)
            .map_err(|e| StockServiceError::ExternalError(format!("Parse error: {}", e)))?;

        let price = quote.global_quote.price.parse::<Decimal>()
            .map_err(|_| StockServiceError::ExternalError("Invalid price format".to_string()))?;

        let volume = quote.global_quote.volume.parse::<u64>()
            .map_err(|_| StockServiceError::ExternalError("Invalid volume format".to_string()))?;

        let change = quote.global_quote.change.parse::<Decimal>().ok();
        
        let change_percent = quote.global_quote.change_percent
            .trim_end_matches('%')
            .parse::<Decimal>().ok();

        let stock_price = StockPrice {
            symbol: symbol.clone(),
            price,
            volume,
            timestamp: chrono::Utc::now(),
            change,
            change_percent,
        };

        // Cache for 1 minute
        self.cache_price(symbol, stock_price.clone(), 60);

        Ok(stock_price)
    }

    async fn get_historical_data(&self, symbol: &Symbol, period: &str) -> Result<Vec<StockPrice>, StockServiceError> {
        let function = match period {
            "daily" | "1d" => "TIME_SERIES_DAILY",
            "weekly" | "1w" => "TIME_SERIES_WEEKLY",
            "monthly" | "1m" => "TIME_SERIES_MONTHLY",
            _ => return Err(StockServiceError::InvalidPeriod(period.to_string())),
        };

        let symbol_str = symbol.to_string();
        let params = [("symbol", symbol_str.as_str())];
        let json = self.make_request(function, &params).await?;

        let time_series: AlphaVantageTimeSeries = serde_json::from_value(json)
            .map_err(|e| StockServiceError::ExternalError(format!("Parse error: {}", e)))?;

        let mut prices = Vec::new();

        if let Some(daily_data) = time_series.time_series {
            for (date_str, data) in daily_data {
                let timestamp = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|_| StockServiceError::ExternalError("Invalid date format".to_string()))?
                    .and_hms_opt(16, 0, 0) // Assume market close at 4 PM
                    .ok_or_else(|| StockServiceError::ExternalError("Invalid time".to_string()))?
                    .and_utc();

                let price = data.close.parse::<Decimal>()
                    .map_err(|_| StockServiceError::ExternalError("Invalid price format".to_string()))?;

                let volume = data.volume.parse::<u64>()
                    .map_err(|_| StockServiceError::ExternalError("Invalid volume format".to_string()))?;

                prices.push(StockPrice {
                    symbol: symbol.clone(),
                    price,
                    volume,
                    timestamp,
                    change: None,
                    change_percent: None,
                });
            }
        }

        // Sort by timestamp descending (newest first)
        prices.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(prices)
    }

    async fn get_market_status(&self, market: &str) -> Result<MarketStatus, StockServiceError> {
        // Alpha Vantage doesn't have a direct market status endpoint
        // We'll provide a basic implementation based on market hours
        let now = chrono::Utc::now();
        let weekday = now.weekday();

        let (is_open, next_open, next_close, timezone) = match market.to_uppercase().as_str() {
            "NYSE" | "NASDAQ" | "US" => {
                let is_weekend = matches!(weekday, chrono::Weekday::Sat | chrono::Weekday::Sun);
                let hour = now.hour();
                
                let is_trading_hours = !is_weekend && hour >= 14 && hour < 21; // 9:30 AM - 4:00 PM EST in UTC
                
                (is_trading_hours, None, None, "America/New_York".to_string())
            },
            _ => return Err(StockServiceError::ExternalError(format!("Unsupported market: {}", market))),
        };

        Ok(MarketStatus {
            market: market.to_string(),
            is_open,
            next_open,
            next_close,
            timezone,
        })
    }

    async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>, StockServiceError> {
        let params = [("keywords", query)];
        let json = self.make_request("SYMBOL_SEARCH", &params).await?;

        let matches = json.get("bestMatches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| StockServiceError::ExternalError("No matches in response".to_string()))?;

        let mut results = Vec::new();

        for match_obj in matches {
            if let Some(symbol_str) = match_obj.get("1. symbol").and_then(|v| v.as_str()) {
                if let Ok(symbol) = Symbol::new(symbol_str) {
                    let name = match_obj.get("2. name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let market = match_obj.get("4. region")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let currency = match_obj.get("8. currency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("USD")
                        .to_string();

                    results.push(SymbolInfo {
                        symbol,
                        name,
                        market,
                        sector: None, // Alpha Vantage search doesn't provide sector info
                        currency,
                    });
                }
            }
        }

        Ok(results)
    }
}

/// Mock market data service for testing
pub struct MockMarketDataService {
    pub should_fail: bool,
    pub mock_price: Decimal,
}

impl MockMarketDataService {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            mock_price: Decimal::new(10000, 2), // $100.00
        }
    }

    pub fn new_failing() -> Self {
        Self {
            should_fail: true,
            mock_price: Decimal::new(10000, 2),
        }
    }

    pub fn with_price(mut self, price: Decimal) -> Self {
        self.mock_price = price;
        self
    }
}

#[async_trait]
impl StockDataSvc for MockMarketDataService {
    async fn get_real_time_price(&self, symbol: &Symbol) -> Result<StockPrice, StockServiceError> {
        if self.should_fail {
            return Err(StockServiceError::ExternalError("Mock failure".to_string()));
        }

        Ok(StockPrice {
            symbol: symbol.clone(),
            price: self.mock_price,
            volume: 1000000,
            timestamp: chrono::Utc::now(),
            change: Some(Decimal::new(250, 2)), // $2.50
            change_percent: Some(Decimal::new(256, 2)), // 2.56%
        })
    }

    async fn get_historical_data(&self, symbol: &Symbol, _period: &str) -> Result<Vec<StockPrice>, StockServiceError> {
        if self.should_fail {
            return Err(StockServiceError::ExternalError("Mock failure".to_string()));
        }

        let mut prices = Vec::new();
        let now = chrono::Utc::now();

        // Generate 30 days of mock data
        for i in 0..30 {
            let timestamp = now - chrono::Duration::days(i);
            let price_variation = Decimal::new((i * 10) as i64, 2); // Small daily variations
            
            prices.push(StockPrice {
                symbol: symbol.clone(),
                price: self.mock_price + price_variation,
                volume: 1000000 + (i as u64 * 10000),
                timestamp,
                change: None,
                change_percent: None,
            });
        }

        Ok(prices)
    }

    async fn get_market_status(&self, market: &str) -> Result<MarketStatus, StockServiceError> {
        if self.should_fail {
            return Err(StockServiceError::ExternalError("Mock failure".to_string()));
        }

        Ok(MarketStatus {
            market: market.to_string(),
            is_open: true,
            next_open: None,
            next_close: Some(chrono::Utc::now() + chrono::Duration::hours(8)),
            timezone: "America/New_York".to_string(),
        })
    }

    async fn search_symbols(&self, query: &str) -> Result<Vec<SymbolInfo>, StockServiceError> {
        if self.should_fail {
            return Err(StockServiceError::ExternalError("Mock failure".to_string()));
        }

        // Return mock search results
        let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA"];
        let mut results = Vec::new();

        for symbol_str in symbols {
            if symbol_str.to_lowercase().contains(&query.to_lowercase()) {
                if let Ok(symbol) = Symbol::new(symbol_str) {
                    results.push(SymbolInfo {
                        symbol,
                        name: format!("{} Corporation", symbol_str),
                        market: "NASDAQ".to_string(),
                        sector: Some("Technology".to_string()),
                        currency: "USD".to_string(),
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_market_data_service() {
        let service = MockMarketDataService::new().with_price(Decimal::new(15050, 2)); // $150.50
        let symbol = Symbol::new("AAPL").unwrap();

        let price = service.get_real_time_price(&symbol).await.unwrap();
        assert_eq!(price.price, Decimal::new(15050, 2));
        assert_eq!(price.symbol, symbol);

        let historical = service.get_historical_data(&symbol, "daily").await.unwrap();
        assert_eq!(historical.len(), 30);

        let market_status = service.get_market_status("NYSE").await.unwrap();
        assert_eq!(market_status.market, "NYSE");
        assert!(market_status.is_open);

        let search_results = service.search_symbols("AAPL").await.unwrap();
        assert!(!search_results.is_empty());
        assert_eq!(search_results[0].symbol, "AAPL");
    }

    #[tokio::test] 
    async fn test_price_caching() {
        let service = AlphaVantageService::new(MarketDataConfig {
            api_key: "test".to_string(),
            base_url: "https://test.com".to_string(),
            rate_limit: 5,
            timeout_seconds: 30,
        });

        let symbol = Symbol::new("TEST").unwrap();
        let price = StockPrice {
            symbol: symbol.clone(),
            price: Decimal::new(100, 0),
            volume: 1000,
            timestamp: chrono::Utc::now(),
            change: None,
            change_percent: None,
        };

        // Cache the price
        service.cache_price(&symbol, price.clone(), 60);

        // Should retrieve from cache
        let cached = service.get_cached_price(&symbol).unwrap();
        assert_eq!(cached.price, price.price);
    }
}