use super::rest::TradingViewRestClient;
use super::scanner::TradingViewScanner;
use super::types::{MarketDataError, TradingViewConfig, TradingViewResponse};
use super::websocket::TradingViewWebSocketHandler;
use crate::config::Config;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use std::sync::Arc;

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
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(
                super::cache::TradingViewCache::new(),
            )),
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
        validate_ranking_report_dates(&response)?;
        let total = response.total_count.unwrap_or(response.data.len() as i32);
        let results = self.scanner.process_trading_view_response(response);

        Ok((results, total))
    }

    /// Fetch one ranking page with exactly one provider attempt. The bounded
    /// rankings decorator owns retries and the total request deadline.
    pub async fn fetch_eps_growth_ranking_once(
        &self,
        skip: i32,
        limit: i32,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> Result<(Vec<StockScreeningResult>, i32), MarketDataError> {
        let payload = self
            .scanner
            .build_screener_request_with_params(skip, limit, country, sector, sort_by);

        let response = self
            .rest_client
            .execute_custom_request_once(payload)
            .await?;
        validate_ranking_report_dates(&response)?;
        let total = resolve_market_rankings_total(response.total_count, skip, response.data.len())?;
        let results = self.scanner.process_trading_view_response(response);

        Ok((results, total))
    }

    /// Fetch specific symbols concurrently
    pub async fn fetch_symbols_concurrent(
        &self,
        symbols: Vec<String>,
    ) -> Result<Vec<EPSGrowthData>, MarketDataError> {
        let payload = self.scanner.build_symbols_request(symbols);
        let response = self.rest_client.execute_custom_request(payload, 3).await?;
        let screening_results = self.scanner.process_trading_view_response(response);

        // Convert screening results to EPS growth data (simplified conversion for now)
        let eps_data = screening_results
            .into_iter()
            .map(|s| {
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
                    next_earnings_date: s
                        .next_earnings_date
                        .and_then(super::report_dates::date_string),
                    last_earnings_date: s
                        .last_earnings_date
                        .and_then(super::report_dates::date_string),
                }
            })
            .collect();

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

/// Normal exclusion happens in the scanner query. If a provider ignores that
/// condition or returns malformed timestamps, reject the page instead of sending
/// undated cards or silently shortening a page with incorrect totals.
fn validate_ranking_report_dates(response: &TradingViewResponse) -> Result<(), MarketDataError> {
    let now = chrono::Utc::now();
    for stock in &response.data {
        let dates = super::report_dates::extract_report_dates(&stock.d, now);
        if dates.next.or(dates.last).is_none() {
            return Err(MarketDataError::ValidationError(format!(
                "Provider ranking contains no usable report date for {}",
                stock.s
            )));
        }
    }
    Ok(())
}

fn resolve_market_rankings_total(
    provider_total: Option<i32>,
    skip: i32,
    page_len: usize,
) -> Result<i32, MarketDataError> {
    if let Some(total) = provider_total {
        return Ok(total);
    }

    let page_len = i32::try_from(page_len).map_err(|_| {
        MarketDataError::ValidationError("Provider page length is unsupported".to_string())
    })?;
    skip.checked_add(page_len).ok_or_else(|| {
        MarketDataError::ValidationError("Provider total is unsupported".to_string())
    })
}

#[cfg(test)]
mod a2_5_tests {
    use super::super::types::{StockDataField, TradingViewStock};
    use super::*;

    fn report_page(index: usize, value: StockDataField) -> TradingViewResponse {
        let mut fields = vec![StockDataField::Null; 39];
        fields[index] = value;
        TradingViewResponse {
            data: vec![TradingViewStock {
                s: "TEST:DATED".into(),
                d: fields,
            }],
            total_count: Some(1),
        }
    }

    #[test]
    fn rankings_accept_each_release_field_including_previous_only_today_and_passed() {
        for index in [32, 33, 34, 36, 37, 38] {
            for timestamp in [
                1_785_283_200,
                chrono::Utc::now()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .timestamp(),
                chrono::Utc::now().timestamp() + 86400,
            ] {
                assert!(validate_ranking_report_dates(&report_page(
                    index,
                    StockDataField::Integer(timestamp)
                ))
                .is_ok());
            }
        }
        assert!(validate_ranking_report_dates(&TradingViewResponse {
            data: vec![],
            total_count: Some(0)
        })
        .is_ok());
    }

    #[test]
    fn rankings_reject_missing_or_malformed_dates_instead_of_shortening_pages() {
        for invalid in [
            StockDataField::Null,
            StockDataField::Integer(0),
            StockDataField::Integer(-1),
            StockDataField::Integer(1_785_283_200_000),
            StockDataField::Number(1_785_283_200.5),
            StockDataField::String("invalid".into()),
        ] {
            let mut page = report_page(33, StockDataField::Integer(1_793_145_600));
            page.data.extend(report_page(33, invalid).data);
            page.total_count = Some(2);
            assert!(matches!(
                validate_ranking_report_dates(&page),
                Err(MarketDataError::ValidationError(_))
            ));
            assert_eq!(page.data.len(), 2);
            assert_eq!(page.total_count, Some(2));
        }
    }

    #[test]
    fn a2_5_missing_provider_total_preserves_current_page_extent() {
        assert_eq!(resolve_market_rankings_total(None, 99, 6).unwrap(), 105);
        assert_eq!(
            resolve_market_rankings_total(Some(250), 99, 6).unwrap(),
            250
        );
    }
}
