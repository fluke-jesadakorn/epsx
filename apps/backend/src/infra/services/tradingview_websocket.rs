// TradingView WebSocket service for real-time EPS data
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::dom::entities::market_data::MarketDataError;
use crate::infra::services::tradingview::FrontendEPSData;

/// TradingView WebSocket message structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewMessage {
    pub m: String, // method
    pub p: Vec<Value>, // parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<i64>, // timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ms: Option<i64>, // timestamp milliseconds
}

/// EPS data extracted from WebSocket
#[derive(Debug, Clone)]
pub struct EPSWebSocketData {
    pub symbol: String,
    pub current_eps: f64,
    pub quarterly_eps: f64,
    pub historical_eps: Vec<f64>,
    pub earnings_per_share_basic_ttm: f64,
    pub market_cap_basic: f64,
    pub price_current: f64,
    pub volume: f64,
    pub sector: String,
    pub country: String,
    pub company_name: String,
}

/// TradingView WebSocket service
pub struct TradingViewWebSocketService {
    websocket_url: String,
    session_id: String,
    symbols: Vec<String>,
    eps_data_cache: HashMap<String, EPSWebSocketData>,
}

impl TradingViewWebSocketService {
    /// Create new WebSocket service
    pub fn new() -> Self {
        let session_id = format!("cs_{}", 
            Uuid::new_v4().to_string().replace("-", "").chars().take(12).collect::<String>()
        );
        
        Self {
            websocket_url: "wss://data.tradingview.com/socket.io/websocket".to_string(),
            session_id,
            symbols: Vec::new(),
            eps_data_cache: HashMap::new(),
        }
    }

    /// Connect to TradingView WebSocket and start receiving EPS data
    pub async fn connect_and_fetch_eps_data(&mut self, symbols: Vec<String>) -> Result<Vec<EPSWebSocketData>, MarketDataError> {
        info!("Connecting to TradingView WebSocket for {} symbols", symbols.len());
        self.symbols = symbols.clone();

        match self.connect_websocket().await {
            Ok(eps_data) => {
                info!("Successfully collected EPS data for {} symbols", eps_data.len());
                Ok(eps_data)
            }
            Err(e) => {
                error!("WebSocket connection failed: {}", e);
                Err(e)
            }
        }
    }

    /// Internal method to handle WebSocket connection
    async fn connect_websocket(&mut self) -> Result<Vec<EPSWebSocketData>, MarketDataError> {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.websocket_url).await
            .map_err(|e| MarketDataError::NetworkError(format!("WebSocket connection failed: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // Step 1: Create chart session
        let session_msg = self.create_chart_session_message();
        let formatted_msg = self.format_tradingview_message(&session_msg);
        write.send(Message::Text(formatted_msg)).await
            .map_err(|e| MarketDataError::NetworkError(format!("Failed to send session message: {}", e)))?;

        info!("Sent chart session creation message");

        // Process symbols in batches
        let batch_size = 5; // Process 5 symbols at a time to avoid overwhelming the service
        for symbol_batch in self.symbols.chunks(batch_size) {
            for symbol in symbol_batch {
                // Step 2: Create series for this symbol
                let series_id = format!("sds_{}", 
                    Uuid::new_v4().to_string().replace("-", "").chars().take(8).collect::<String>()
                );
                
                let series_msg = self.create_series_message(&series_id, symbol);
                let formatted_series = self.format_tradingview_message(&series_msg);
                write.send(Message::Text(formatted_series)).await
                    .map_err(|e| MarketDataError::NetworkError(format!("Failed to send series message: {}", e)))?;

                debug!("Sent create_series for symbol: {}", symbol);

                // Step 3: Request quote data
                let quote_session_id = format!("qs_{}", 
                    Uuid::new_v4().to_string().replace("-", "").chars().take(10).collect::<String>()
                );
                
                let quote_msg = self.create_quote_session_message(&quote_session_id, symbol);
                let formatted_quote = self.format_tradingview_message(&quote_msg);
                write.send(Message::Text(formatted_quote)).await
                    .map_err(|e| MarketDataError::NetworkError(format!("Failed to send quote message: {}", e)))?;

                debug!("Sent quote session for symbol: {}", symbol);
            }

            // Small delay between batches
            sleep(Duration::from_millis(100)).await;
        }

        // Step 4: Listen for responses
        let mut collected_data = Vec::new();
        let mut timeout_count = 0;
        let max_timeouts = 50; // Allow up to 5 seconds of waiting

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Some(eps_data) = self.process_websocket_message(&text) {
                                collected_data.push(eps_data);
                                info!("Collected EPS data for symbol, total: {}", collected_data.len());
                                
                                // If we have data for all symbols, we can stop
                                if collected_data.len() >= self.symbols.len() {
                                    break;
                                }
                            }
                            timeout_count = 0; // Reset timeout on successful message
                        }
                        Some(Ok(Message::Close(_))) => {
                            warn!("WebSocket connection closed");
                            break;
                        }
                        Some(Ok(Message::Binary(_))) => {
                            debug!("Received binary message, ignoring");
                        }
                        Some(Ok(Message::Ping(_))) => {
                            debug!("Received ping message");
                        }
                        Some(Ok(Message::Pong(_))) => {
                            debug!("Received pong message");
                        }
                        Some(Ok(Message::Frame(_))) => {
                            debug!("Received frame message, ignoring");
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            return Err(MarketDataError::NetworkError(format!("WebSocket error: {}", e)));
                        }
                        None => {
                            warn!("WebSocket stream ended");
                            break;
                        }
                    }
                }
                _ = sleep(Duration::from_millis(100)) => {
                    timeout_count += 1;
                    if timeout_count > max_timeouts {
                        info!("WebSocket timeout reached, stopping collection");
                        break;
                    }
                }
            }
        }

        if collected_data.is_empty() {
            warn!("No EPS data collected from WebSocket");
        }

        Ok(collected_data)
    }

    /// Create chart session message
    fn create_chart_session_message(&self) -> TradingViewMessage {
        TradingViewMessage {
            m: "chart_create_session".to_string(),
            p: vec![json!(self.session_id), json!("")],
            t: None,
            t_ms: None,
        }
    }

    /// Create series message for a symbol
    fn create_series_message(&self, series_id: &str, _symbol: &str) -> TradingViewMessage {
        TradingViewMessage {
            m: "create_series".to_string(),
            p: vec![
                json!(self.session_id),
                json!(series_id),
                json!("s1"),
                json!(format!("sds_sym_{}", series_id.chars().take(4).collect::<String>())),
                json!("1D"),
                json!(300),
                json!("")
            ],
            t: None,
            t_ms: None,
        }
    }

    /// Create quote session message
    fn create_quote_session_message(&self, quote_id: &str, _symbol: &str) -> TradingViewMessage {
        TradingViewMessage {
            m: "quote_create_session".to_string(),
            p: vec![json!(quote_id)],
            t: None,
            t_ms: None,
        }
    }

    /// Format message in TradingView protocol format: ~m~{length}~m~{JSON}
    fn format_tradingview_message(&self, msg: &TradingViewMessage) -> String {
        let json_str = serde_json::to_string(msg).unwrap_or_default();
        let length = json_str.len();
        format!("~m~{}~m~{}", length, json_str)
    }

    /// Process incoming WebSocket message and extract EPS data
    fn process_websocket_message(&mut self, text: &str) -> Option<EPSWebSocketData> {
        // Parse TradingView protocol: ~m~{length}~m~{JSON}
        if !text.starts_with("~m~") {
            return None;
        }

        let parts: Vec<&str> = text.splitn(4, "~m~").collect();
        if parts.len() < 4 {
            return None;
        }

        let json_part = parts[3];
        let parsed: Result<TradingViewMessage, _> = serde_json::from_str(json_part);
        
        match parsed {
            Ok(msg) => {
                debug!("Received message type: {}", msg.m);
                
                match msg.m.as_str() {
                    "symbol_resolved" => self.extract_eps_from_symbol_resolved(msg),
                    "qsd" => self.extract_eps_from_qsd(msg),
                    "timescale_update" => self.extract_historical_data(msg),
                    _ => {
                        debug!("Unhandled message type: {}", msg.m);
                        None
                    }
                }
            }
            Err(e) => {
                debug!("Failed to parse WebSocket message: {}", e);
                None
            }
        }
    }

    /// Extract EPS data from symbol_resolved message
    fn extract_eps_from_symbol_resolved(&mut self, msg: TradingViewMessage) -> Option<EPSWebSocketData> {
        if msg.p.len() >= 3 {
            if let Some(symbol_data) = msg.p.get(2) {
                return self.parse_symbol_data_for_eps(symbol_data);
            }
        }
        None
    }

    /// Extract EPS data from qsd (quote session data) message  
    fn extract_eps_from_qsd(&mut self, msg: TradingViewMessage) -> Option<EPSWebSocketData> {
        if msg.p.len() >= 2 {
            if let Some(quote_data) = msg.p.get(1).and_then(|p| p.get("v")) {
                return self.parse_quote_data_for_eps(quote_data);
            }
        }
        None
    }

    /// Extract historical EPS data from timescale_update
    fn extract_historical_data(&mut self, msg: TradingViewMessage) -> Option<EPSWebSocketData> {
        // Parse timescale_update messages that contain historical EPS data
        // Format: {"i": index, "v": [timestamp, eps_value, ...other_fields]}
        
        if msg.p.len() >= 2 {
            // Get the update data array
            if let Some(update_data) = msg.p.get(1).and_then(|p| p.get("st")).and_then(|st| st.as_array()) {
                let mut historical_eps = Vec::new();
                let mut symbol = String::new();
                let mut latest_eps = 0.0;
                
                // Process each data point in the timescale update
                for data_point in update_data {
                    if let Some(v_array) = data_point.get("v").and_then(|v| v.as_array()) {
                        // Extract EPS value from the array
                        // Based on example: {"i":228,"v":[1744941600.0,6.6102,5.992452,1743379200.0,1744934400000.0,6.61,...]}
                        // The EPS value appears to be at index 1 (6.6102) and 5 (6.61)
                        if let Some(eps_value) = v_array.get(1).and_then(|v| v.as_f64()) {
                            historical_eps.push(eps_value);
                            latest_eps = eps_value; // Keep track of the most recent EPS
                        }
                    }
                }
                
                // Extract symbol from session or use a default
                if let Some(session_id) = msg.p.get(0).and_then(|p| p.as_str()) {
                    symbol = session_id.to_string();
                }
                
                if !historical_eps.is_empty() {
                    debug!("Extracted {} historical EPS values: {:?}", historical_eps.len(), historical_eps);
                    
                    return Some(EPSWebSocketData {
                        symbol,
                        current_eps: latest_eps,
                        quarterly_eps: latest_eps,
                        historical_eps,
                        earnings_per_share_basic_ttm: latest_eps,
                        market_cap_basic: 0.0, // Will be filled from other messages
                        price_current: 0.0, // Will be filled from other messages
                        volume: 0.0, // Will be filled from other messages
                        sector: String::new(), // Will be filled from other messages
                        country: String::new(), // Will be filled from other messages
                        company_name: String::new(), // Will be filled from other messages
                    });
                }
            }
        }
        
        None
    }

    /// Parse symbol data to extract EPS information
    fn parse_symbol_data_for_eps(&self, symbol_data: &Value) -> Option<EPSWebSocketData> {
        let name = symbol_data.get("name")?.as_str().unwrap_or("").to_string();
        let description = symbol_data.get("description")?.as_str().unwrap_or("").to_string();
        let country = symbol_data.get("country")?.as_str().unwrap_or("").to_string();
        let sector = symbol_data.get("sector")?.as_str().unwrap_or("").to_string();
        
        Some(EPSWebSocketData {
            symbol: name,
            current_eps: 0.0, // Will be updated from qsd message
            quarterly_eps: 0.0,
            historical_eps: Vec::new(),
            earnings_per_share_basic_ttm: 0.0,
            market_cap_basic: 0.0,
            price_current: 0.0,
            volume: 0.0,
            sector,
            country,
            company_name: description,
        })
    }

    /// Parse quote data to extract EPS values
    fn parse_quote_data_for_eps(&self, quote_data: &Value) -> Option<EPSWebSocketData> {
        let earnings_per_share_basic_ttm = quote_data.get("earnings_per_share_basic_ttm")
            ?.as_f64().unwrap_or(0.0);
        let earnings_per_share_fq = quote_data.get("earnings_per_share_fq")
            ?.as_f64().unwrap_or(0.0);
        let market_cap_basic = quote_data.get("market_cap_basic")
            ?.as_f64().unwrap_or(0.0);
        let volume = quote_data.get("volume")?.as_f64().unwrap_or(0.0);
        let lp = quote_data.get("lp")?.as_f64().unwrap_or(0.0); // last price
        let sector = quote_data.get("sector")?.as_str().unwrap_or("").to_string();
        let country_code = quote_data.get("country_code")?.as_str().unwrap_or("").to_string();
        let description = quote_data.get("description")?.as_str().unwrap_or("").to_string();
        let symbol_primary = quote_data.get("symbol-primaryname")?.as_str().unwrap_or("");
        let symbol = symbol_primary.split(':').nth(1).unwrap_or(symbol_primary).to_string();

        Some(EPSWebSocketData {
            symbol,
            current_eps: earnings_per_share_basic_ttm,
            quarterly_eps: earnings_per_share_fq,
            historical_eps: vec![earnings_per_share_basic_ttm], // Start with current EPS
            earnings_per_share_basic_ttm,
            market_cap_basic,
            price_current: lp,
            volume,
            sector,
            country: country_code,
            company_name: description,
        })
    }

    /// Convert WebSocket EPS data to frontend format
    pub fn convert_to_frontend_format(&self, websocket_data: Vec<EPSWebSocketData>) -> Vec<FrontendEPSData> {
        websocket_data.into_iter().map(|ws_data| {
            let qoq_growth = self.calculate_qoq_growth(&ws_data.historical_eps);
            let ranking_score = self.calculate_ranking_score(
                ws_data.current_eps, 
                qoq_growth, 
                ws_data.market_cap_basic, 
                ws_data.price_current
            );

            FrontendEPSData {
                id: Uuid::new_v4().to_string(),
                symbol: ws_data.symbol,
                company_name: ws_data.company_name,
                current_eps: ws_data.current_eps,
                qoq_growth,
                market_cap: ws_data.market_cap_basic as i64,
                price_current: ws_data.price_current,
                volume: ws_data.volume as i64,
                country: ws_data.country,
                sector: ws_data.sector,
                ranking_score,
            }
        }).collect()
    }

    /// Calculate quarter-over-quarter growth from historical EPS
    fn calculate_qoq_growth(&self, historical_eps: &[f64]) -> f64 {
        if historical_eps.len() < 2 {
            return 0.0;
        }
        
        let current = historical_eps.last().unwrap_or(&0.0);
        let previous = historical_eps.get(historical_eps.len() - 2).unwrap_or(&0.0);
        
        if *previous == 0.0 {
            return 0.0;
        }
        
        ((current - previous) / previous) * 100.0
    }

    /// Calculate ranking score (same algorithm as TradingView service)
    fn calculate_ranking_score(&self, current_eps: f64, qoq_growth: f64, market_cap: f64, price: f64) -> f64 {
        let eps_weight = 0.3;
        let growth_weight = 0.4;
        let market_cap_weight = 0.2;
        let price_weight = 0.1;

        let eps_score = (current_eps * 10.0).min(100.0).max(0.0);
        let growth_score = (qoq_growth.abs() / 100.0 * 100.0).min(100.0).max(0.0);
        let market_cap_score = (market_cap / 1_000_000_000_000.0 * 100.0).min(100.0).max(0.0);
        let price_score = (price / 1000.0 * 100.0).min(100.0).max(0.0);

        (eps_score * eps_weight + growth_score * growth_weight + 
         market_cap_score * market_cap_weight + price_score * price_weight).round()
    }
}