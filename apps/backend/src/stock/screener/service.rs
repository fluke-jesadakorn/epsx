use std::time::Duration;
use std::sync::Arc;
use mongodb::bson::doc;
use reqwest::{Client, ClientBuilder};
use serde_json::json;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{config::Config, db::DB};
use crate::stock::common::{StockServiceError, TradingViewResponse, TradingViewStock, StockDataField, NumberFormatter, WebSocketClient};
use super::models::{TableDataMetrics, QuoteSessionCreate};

pub struct ScreenerService {
    client: Client,
    db: Arc<DB>,
    ws_client: WebSocketClient,
}

impl ScreenerService {
    pub fn new(config: &Config, db: Arc<DB>) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko)")
            .build()
            .unwrap_or_else(|_| Client::new());

        let ws_client = WebSocketClient::new(
            "wss://data.tradingview.com/socket.io/websocket?from=screener",
            &config.tradingview_auth_token
        );

        Self { 
            client,
            db, 
            ws_client 
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
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        let result = self.db.get_stock_data()
            .find_one(
                doc! { "fetch_date": &today },
                None
            )
            .await
            .map_err(|e| StockServiceError::from(e))?;

        Ok(result.map(|data| data.stocks))
    }

    async fn save_stock_data(&self, stocks: Vec<TableDataMetrics>) -> Result<(), StockServiceError> {
        let stock_data = crate::db::models::StockData::new(stocks);
        
        self.db.get_stock_data()
            .insert_one(&stock_data, None)
            .await
            .map_err(|e| StockServiceError::from(e))?;

        Ok(())
    }

    fn get_request_headers() -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("accept", reqwest::header::HeaderValue::from_static("text/plain, */*; q=0.01"));
        headers.insert("accept-language", reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", reqwest::header::HeaderValue::from_static("application/json"));
        headers.insert("origin", reqwest::header::HeaderValue::from_static("https://www.tradingview.com"));
        headers.insert("referer", reqwest::header::HeaderValue::from_static("https://www.tradingview.com/"));
        headers.insert("sec-ch-ua", reqwest::header::HeaderValue::from_static(r#""Not A(Brand";v="99", "Google Chrome";v="121""#));
        headers.insert("sec-ch-ua-mobile", reqwest::header::HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", reqwest::header::HeaderValue::from_static(r#""macOS""#));
        headers.insert("sec-fetch-dest", reqwest::header::HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", reqwest::header::HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", reqwest::header::HeaderValue::from_static("same-site"));
        headers
    }

    pub async fn fetch_stock_screener_data(&self) -> Result<Vec<TableDataMetrics>, StockServiceError> {
        // First try to get today's data from MongoDB
        if let Some(data) = self.get_todays_data().await? {
            info!("Returning today's cached data from MongoDB");
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
                .post("https://scanner.tradingview.com/global/scan")
                .headers(Self::get_request_headers())
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

        // Save the fetched data to MongoDB
        self.save_stock_data(stocks.clone()).await?;
        
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
                        let score_a = super::models::parse_formatted_number(&a.activity_score);
                        let score_b = super::models::parse_formatted_number(&b.activity_score);
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
