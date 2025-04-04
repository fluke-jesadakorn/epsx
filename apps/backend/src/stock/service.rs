use std::time::Duration;
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HeaderValue}};
use serde_json::json;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{debug, error, info};

use super::{
    models::{TableDataMetrics, TradingViewResponse},
    StockServiceError,
};
use super::models::StockDataField;

fn get_number(data: &[StockDataField], idx: usize) -> f64 {
    match data.get(idx) {
        Some(&StockDataField::Number(n)) => n,
        _ => 0.0
    }
}

pub struct StockService {
    client: Client,
}

impl StockService {
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    fn get_request_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("text/plain, */*; q=0.01"));
        headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("origin", HeaderValue::from_static("https://www.tradingview.com"));
        headers.insert("referer", HeaderValue::from_static("https://www.tradingview.com/"));
        headers.insert("sec-ch-ua", HeaderValue::from_static(r#""Not A(Brand";v="99", "Google Chrome";v="121""#));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static(r#""macOS""#));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-site"));
        headers
    }

    pub async fn fetch_stock_screener_data(&self) -> Result<Vec<TableDataMetrics>, StockServiceError> {
        // Setup retry strategy with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .map(jitter)
            .take(3);

        let result = Retry::spawn(retry_strategy, || async {
            info!("Making request to TradingView API");
            debug!("Attempting to fetch stock screener data from TradingView");
            let response = self
                .client
                .post("https://scanner.tradingview.com/global/scan?label-product=underchart-screener-stock")
                .headers(Self::get_request_headers())
                .json(&json!({
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
            }))
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

        Ok(result
            .data
            .into_iter()
            .map(|stock| {
                // Helper to safely get string from data array with default value
                let get_string = |idx: usize, default: &str| -> String {
                    stock.data
                        .get(idx)
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
                        super::models::PhaseInfo {
                            date: "N/A".to_string(),
                            active: false,
                        },
                        super::models::PhaseStatus {
                            date: "N/A".to_string(),
                            phase_type: super::models::PhaseType::Monitor,
                            active: false,
                        }
                    )
                };

                TableDataMetrics {
                    symbol: stock.symbol.split(':').nth(1).unwrap_or(&stock.symbol).to_string(),
                    name: get_string(0, ""),
                    // Mapping based on columns array in API request
                    value_index: TableDataMetrics::format_number(Some(get_number(&stock.data, 2))), // close
                    growth_rate: TableDataMetrics::format_number(Some(get_number(&stock.data, 3))), // change
                    activity_score: TableDataMetrics::format_large_number(get_number(&stock.data, 6)), // volume
                    market_size: TableDataMetrics::format_large_number(get_number(&stock.data, 8)), // market_cap_basic
                    growth_factor: TableDataMetrics::format_number(Some(get_number(&stock.data, 9))), // price_earnings_ttm
                    sector: get_string(11, "N/A"), // sector
                    country: get_string(12, "N/A"), // country
                    exchange: get_string(13, "N/A"), // exchange
                    currency: get_string(27, "N/A"), // currency
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
            })
            .collect())
    }

    pub async fn fetch_eps_growth_ranking(
        &self,
        limit: Option<i32>,
        skip: Option<i32>,
        sort_by: Option<String>,
    ) -> Result<Vec<TableDataMetrics>, StockServiceError> {
        let mut data = self.fetch_stock_screener_data().await?;

        // Sort data based on the sort_by parameter
        if let Some(sort_field) = sort_by {
            data.sort_by(|a, b| {
                match sort_field.as_str() {
                    "activityScore" => {
                        let score_a = parse_formatted_number(&a.activity_score);
                        let score_b = parse_formatted_number(&b.activity_score);
                        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    _ => { // Default to growth indicator
                        let growth_a = a.growth_indicator.replace('%', "").parse::<f64>().unwrap_or(0.0);
                        let growth_b = b.growth_indicator.replace('%', "").parse::<f64>().unwrap_or(0.0);
                        growth_b.partial_cmp(&growth_a).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }
            });
        }

        // Apply pagination
        let start = skip.unwrap_or(0) as usize;
        let end = start + limit.unwrap_or(10) as usize;

        Ok(data.into_iter().skip(start).take(end - start).collect())
    }
}

// Helper function to parse formatted numbers (e.g., "1.23M" -> 1230000.0)
fn parse_formatted_number(formatted: &str) -> f64 {
    let cleaned = formatted.replace(['T', 'B', 'M'], "");
    let base = cleaned.parse::<f64>().unwrap_or(0.0);
    
    if formatted.contains('T') {
        base * 1e12
    } else if formatted.contains('B') {
        base * 1e9
    } else if formatted.contains('M') {
        base * 1e6
    } else {
        base
    }
}
