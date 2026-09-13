use crate::domain::market_analytics::repository_ports::{
    MarketDataScannerPort, MarketRankingsPage, MarketRankingsProviderPort, MarketRankingsRequest,
};
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use crate::infrastructure::adapters::services::tradingview::types::MarketDataError;
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
use async_trait::async_trait;
use epsx_contracts::errors::{AppError, ErrorKind};
use std::sync::Arc;
use tracing::error;

/// Adapter that implements MarketDataScannerPort for TradingView API service
/// This bridges the domain port interface with the concrete TradingView implementation
pub struct TradingViewAdapter {
    tradingview_service: Arc<TradingViewApiService>,
}

impl TradingViewAdapter {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self {
            tradingview_service,
        }
    }
}

#[async_trait]
impl MarketDataScannerPort for TradingViewAdapter {
    async fn fetch_eps_data(&self, symbol: &str) -> Result<Option<EPSGrowthData>, AppError> {
        // Fetch data for a single symbol using TradingView API
        let symbols = vec![symbol.to_string()];
        let results = self
            .tradingview_service
            .fetch_symbols_concurrent(symbols)
            .await
            .map_err(|e| {
                AppError::external_service_error(format!("TradingView API error: {}", e))
            })?;

        Ok(results.into_iter().next())
    }

    async fn fetch_batch_eps_data(
        &self,
        symbols: &[String],
    ) -> Result<Vec<EPSGrowthData>, AppError> {
        let symbols_vec = symbols.to_vec();
        self.tradingview_service
            .fetch_symbols_concurrent(symbols_vec)
            .await
            .map_err(|e| {
                AppError::external_service_error(format!("TradingView batch API error: {}", e))
            })
    }

    async fn health_check(&self) -> Result<(), AppError> {
        self.tradingview_service
            .test_connections()
            .await
            .map_err(|e| {
                AppError::external_service_error(format!(
                    "TradingView connection test failed: {}",
                    e
                ))
            })
            .map(|_| ())
    }

    async fn get_countries(&self) -> Result<Vec<String>, AppError> {
        // TradingView API doesn't directly provide countries list
        // Return commonly supported countries
        Ok(vec![
            "america".to_string(),
            "europe".to_string(),
            "asia".to_string(),
            "us".to_string(),
            "gb".to_string(),
            "de".to_string(),
            "jp".to_string(),
            "cn".to_string(),
        ])
    }

    async fn get_sectors_by_country(
        &self,
        _country: Option<&str>,
    ) -> Result<Vec<String>, AppError> {
        // TradingView API doesn't directly provide sectors by country
        // Return commonly supported sectors
        Ok(vec![
            "technology".to_string(),
            "finance".to_string(),
            "healthcare".to_string(),
            "energy".to_string(),
            "industrials".to_string(),
            "consumer".to_string(),
            "materials".to_string(),
            "utilities".to_string(),
            "telecom".to_string(),
            "real_estate".to_string(),
        ])
    }

    async fn search_symbols(
        &self,
        _query: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<String>, AppError> {
        // For now, return empty as TradingView API doesn't have direct symbol search
        // This could be enhanced later with actual TradingView symbol search functionality
        Ok(vec![])
    }
}

#[async_trait]
impl MarketRankingsProviderPort for TradingViewAdapter {
    async fn fetch_rankings(
        &self,
        request: MarketRankingsRequest,
    ) -> Result<MarketRankingsPage, AppError> {
        let (items, total) = self
            .tradingview_service
            .fetch_eps_growth_ranking_once(
                request.skip,
                request.limit,
                request.country,
                request.sector,
                request.sort_by,
            )
            .await
            .map_err(map_rankings_error)?;

        Ok(MarketRankingsPage { items, total })
    }
}

fn map_rankings_error(error: MarketDataError) -> AppError {
    error!(error_kind = %market_data_error_kind(&error), "market rankings provider attempt failed");

    match error {
        MarketDataError::HttpStatus(408 | 504) => {
            AppError::new(ErrorKind::TimeoutError, "Market rankings provider timeout")
        }
        MarketDataError::HttpStatus(429 | 500..=599) => AppError::new(
            ErrorKind::ServiceUnavailable,
            "Market rankings provider unavailable",
        ),
        MarketDataError::NetworkError(_) | MarketDataError::ConnectionError(_) => {
            AppError::network_error("Market rankings provider network failure")
        }
        _ => AppError::external_service_error("Market rankings provider failure"),
    }
}

fn market_data_error_kind(error: &MarketDataError) -> &'static str {
    match error {
        MarketDataError::NetworkError(_) => "network",
        MarketDataError::ConnectionError(_) => "connection",
        MarketDataError::ParsingError(_) => "parsing",
        MarketDataError::ExternalApiError(_) => "external_api",
        MarketDataError::HttpStatus(_) => "http_status",
        MarketDataError::SerializationError(_) => "serialization",
        MarketDataError::ValidationError(_) => "validation",
    }
}

#[cfg(test)]
mod a2_5_tests {
    use super::*;

    #[test]
    fn a2_5_http_status_retry_classification_is_explicit() {
        for status in [408, 429, 500, 502, 503, 504, 599] {
            let error = map_rankings_error(MarketDataError::HttpStatus(status));
            assert!(error.is_retryable(), "status {status} must be retryable");
        }

        for status in [400, 401, 403, 404] {
            let error = map_rankings_error(MarketDataError::HttpStatus(status));
            assert!(!error.is_retryable(), "status {status} must be permanent");
            assert_eq!(error.http_status(), 502);
        }
    }
}
