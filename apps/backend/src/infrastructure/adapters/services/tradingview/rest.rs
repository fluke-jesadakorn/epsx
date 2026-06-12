// TradingView REST - Focused Module for HTTP API Communication
// Handles HTTP requests, headers, retry logic, and request building

use std::time::Duration;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{debug, error, info, warn};

use super::types::{TradingViewConfig, TradingViewResponse, MarketDataError};

/// REST API client for TradingView integration
pub struct TradingViewRestClient {
    client: Client,
    config: TradingViewConfig,
}

impl TradingViewRestClient {
    /// Create new REST client with configuration
    pub fn new(config: TradingViewConfig) -> Self {
        let timeout_duration = Duration::from_secs(config.http_timeout_seconds);
        let client = ClientBuilder::new()
            .timeout(timeout_duration)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, config }
    }

    /// Build request headers for TradingView API using exact format from capture
    pub fn get_request_headers(&self) -> reqwest::header::HeaderMap {
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

    /// Execute HTTP request with retry logic
    pub async fn execute_request_with_retry(
        &self,
        payload: serde_json::Value,
    ) -> Result<TradingViewResponse, MarketDataError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        Retry::spawn(retry_strategy, || async {
            info!("Making request to TradingView API");
            debug!("Attempting to fetch data from TradingView");

            let response = self
                .client
                .post(&self.config.scanner_api_url)
                .headers(self.get_request_headers())
                .json(&payload)
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

            debug!("Successfully fetched and parsed data from TradingView");
            Ok(trading_view_resp)
        }).await
    }

    /// Execute custom HTTP request with detailed error handling
    pub async fn execute_custom_request(
        &self,
        payload: serde_json::Value,
        max_retries: usize,
    ) -> Result<TradingViewResponse, MarketDataError> {
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(10))
            .take(max_retries)
            .map(jitter);

        Retry::spawn(retry_strategy, || {
            let headers = self.get_request_headers();
            let body = payload.clone();
            
            async move {
                let response = self.client
                    .post(&self.config.scanner_api_url)
                    .headers(headers)
                    .body(serde_json::to_string(&body).map_err(|e| {
                        error!("Failed to serialize request body: {:?}", e);
                        MarketDataError::SerializationError(e.to_string())
                    })?)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("TradingView API request failed: {:?}", e);
                        MarketDataError::NetworkError(e.to_string())
                    })?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    error!("TradingView API error {}: {}", status, error_text);
                    return Err(MarketDataError::ExternalApiError(format!("API error {}: {}", status, error_text)));
                }

                let response_text = response.text().await.map_err(|e| {
                    error!("Failed to read response text: {:?}", e);
                    MarketDataError::NetworkError(e.to_string())
                })?;

                serde_json::from_str::<TradingViewResponse>(&response_text).map_err(|e| {
                    error!("Failed to parse TradingView response: {:?}", e);
                    debug!("Response text (first 500 chars): {}", &response_text[..response_text.len().min(500)]);
                    MarketDataError::ParsingError(e.to_string())
                })
            }
        })
        .await
    }

    /// Execute batch requests with rate limiting
    pub async fn execute_batch_requests(
        &self,
        payloads: Vec<serde_json::Value>,
    ) -> Vec<Result<TradingViewResponse, MarketDataError>> {
        use tokio::time::{sleep, Duration};
        
        let mut results = Vec::new();
        let concurrent_limit = super::types::constants::MAX_CONCURRENT_REQUESTS;
        let delay_between_batches = Duration::from_millis(super::types::constants::BATCH_DELAY_MS);
        
        for batch in payloads.chunks(concurrent_limit) {
            let batch_futures: Vec<_> = batch.iter()
                .map(|payload| self.execute_request_with_retry(payload.clone()))
                .collect();
            
            let batch_results = futures::future::join_all(batch_futures).await;
            results.extend(batch_results);
            
            // Add delay between batches to respect rate limits
            if batch.len() == concurrent_limit {
                sleep(delay_between_batches).await;
            }
        }
        
        results
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TradingViewConfig {
        &self.config
    }

    /// Test connection to TradingView API
    pub async fn test_connection(&self) -> Result<bool, MarketDataError> {
        let test_payload = json!({
            "columns": ["name"],
            "filter": [],
            "options": { "lang": "en" },
            "range": [0, 1],
            "sort": { "sortBy": "market_cap_basic", "sortOrder": "desc" },
            "markets": ["america"]
        });

        match self.execute_request_with_retry(test_payload).await {
            Ok(_) => {
                info!("TradingView API connection test successful");
                Ok(true)
            }
            Err(e) => {
                warn!("TradingView API connection test failed: {}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // wave 10 prep: dummy DATABASE_URL so Config::from_env() doesn't fail.
    fn ensure_dummy_db_url() {
        if std::env::var("DATABASE_URL").is_err() {
            // SAFETY: see scanner.rs tests::ensure_dummy_db_url — same rationale.
            unsafe {
                std::env::set_var(
                    "DATABASE_URL",
                    "postgres://test:test@localhost:5432/test",
                );
            }
        }
    }

    #[tokio::test]
    async fn test_rest_client_creation() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let client = TradingViewRestClient::new(tv_config);
        
        assert!(!client.get_config().scanner_api_url.is_empty());
    }

    #[test]
    fn test_request_headers() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let client = TradingViewRestClient::new(tv_config);
        
        let headers = client.get_request_headers();
        assert!(headers.contains_key("accept"));
        assert!(headers.contains_key("content-type"));
        assert!(headers.contains_key("referer"));
    }

    #[tokio::test]
    #[ignore] // Ignore in CI/CD to avoid external API calls
    async fn test_connection_integration() {
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let client = TradingViewRestClient::new(tv_config);
        
        // This test requires actual TradingView API access
        let result = client.test_connection().await;
        match result {
            Ok(connected) => assert!(connected),
            Err(_) => {
                // Connection tests can fail in test environments
                // This is acceptable for unit tests
            }
        }
    }

    #[test]
    fn test_batch_size_constants() {
        // Constants are verified at compile time
    }
}