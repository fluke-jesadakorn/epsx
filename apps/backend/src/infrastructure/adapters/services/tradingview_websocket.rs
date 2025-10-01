// TradingView WebSocket service for real-time EPS data
use uuid::Uuid;
use std::time::Duration;

use tokio::time::sleep;

use tokio_tungstenite::{ connect_async, tungstenite::protocol::Message };

use futures_util::{ SinkExt, StreamExt };

use serde::{ Deserialize, Serialize };

use serde_json::{ json, Value };

use tracing::{ debug, info, warn };

use chrono::{Datelike, TimeZone};

use http;

use base64::{self, prelude::*};

use rand;

use url;


use crate::core::errors::AppError;

/// Frontend EPS data format for client consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendEPSData {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub company_name: String,
    pub current_eps: f64,
    pub previous_eps: f64,
    pub growth_rate: f64,
    pub qoq_growth: f64,
    pub market_cap: i64,
    pub price_current: f64,
    pub volume: i64,
    pub country: String,
    pub sector: String,
    pub ranking_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}


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

/// Quarterly EPS data point (matching Node.js structure exactly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyEPSData {
  pub quarter_number: usize,
  pub period: String, // "2024-Q3", "2025-Q1", etc.
  pub actual_eps: f64,
  pub timestamp: i64, // earnings timestamp (seconds)
  pub estimated_eps: Option<f64>,
  pub is_reported: bool,
  pub beat_estimate: Option<bool>,
  #[serde(rename = "type")]
  pub eps_type: String, // "st4_earnings_study"
  pub source: String,
  pub quarter_end_date: Option<String>,
  pub estimated_earnings_date: Option<i64>, // earnings announcement timestamp
  pub price_data: Option<PriceImpactData>,
  // Legacy fields for backward compatibility
  pub eps: f64,
  pub quarter_name: String,
}

/// Price impact data around earnings announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpactData {
  pub pre_earnings_price: f64,
  pub post_earnings_price: f64, 
  pub price_change: f64,
  pub percent_change: f64,
  pub earnings_impact: String, // "positive" or "negative"
  pub days_before: i32,
  pub days_after: i32,
  pub volume_before: i64,
  pub volume_after: i64,
  pub volume_change: String, // "increased" or "decreased"
  pub data_quality: String, // "excellent", "good", "fair"
}

/// Price data point extracted from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
  pub timestamp: i64,
  pub date: String,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

/// EPS data extracted from WebSocket with price correlation
#[derive(Debug, Clone, Serialize)]
pub struct EPSWebSocketData {
  pub symbol: String,
  pub current_eps: f64,
  pub quarterly_eps: f64,
  pub historical_eps: Vec<f64>,
  pub quarterly_data: Vec<QuarterlyEPSData>, // Real quarterly progression from study data
  pub price_data: Vec<PriceData>, // OHLCV price data for correlation
  pub earnings_per_share_basic_ttm: f64,
  pub market_cap_basic: f64,
  pub price_current: f64,
  pub volume: f64,
  pub sector: String,
  pub country: String,
  pub company_name: String,
}

/// TradingView WebSocket service with exact Node.js devtools logic
pub struct TradingViewWebSocketService {
  websocket_url: String,
  #[allow(dead_code)]
  session_id: String,
  symbols: Vec<String>,
  price_data: std::collections::HashMap<i64, PriceData>,
  #[allow(dead_code)]
  message_log: Vec<String>,
  debug: bool,
  // State tracking like Node.js
  chart_session: Option<String>,
  quote_session: Option<String>,
  series_ready: bool,
  symbol_resolved: bool,
  extracted_eps_data: Option<EPSWebSocketData>,
}

impl TradingViewWebSocketService {
  /// Create new WebSocket service
  pub fn new() -> Self {
    let session_id = format!(
      "cs_{}",
      Uuid::new_v4()
        .to_string()
        .replace("-", "")
        .chars()
        .take(12)
        .collect::<String>()
    );

    Self {
      websocket_url: "wss://data.tradingview.com/socket.io/websocket".to_string(),
      session_id,
      symbols: Vec::new(),
      price_data: std::collections::HashMap::new(),
      message_log: Vec::new(),
      debug: true,
      chart_session: None,
      quote_session: None,
      series_ready: false,
      symbol_resolved: false,
      extracted_eps_data: None,
    }
  }

  /// Connect to TradingView WebSocket and start receiving EPS data
  pub async fn connect_and_fetch_eps_data(
    &mut self,
    symbols: Vec<String>
  ) -> Result<Vec<EPSWebSocketData>, AppError> {
    info!("🚀 Starting TradingView WebSocket connection for {} symbols", symbols.len());
    self.symbols = symbols.clone();

    // Parse URL for WebSocket connection (like Node.js implementation)
    let url = match url::Url::parse(&self.websocket_url) {
      Ok(url) => url,
      Err(e) => {
        warn!("❌ Failed to parse WebSocket URL: {}", e);
        return Err(AppError::network_error(format!("Invalid WebSocket URL: {}", e)));
      }
    };

    // Create WebSocket connection with exact headers from Node.js
    let request = http::Request::builder()
      .method("GET")
      .uri(url.as_str())
      .header("Host", url.host_str().unwrap_or("data.tradingview.com"))
      .header("Connection", "Upgrade")
      .header("Upgrade", "websocket")
      .header("Sec-WebSocket-Version", "13")
      .header("Sec-WebSocket-Key", base64::prelude::BASE64_STANDARD.encode(&rand::random::<[u8; 16]>()))
      .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
      .header("Origin", "https://www.tradingview.com")
      .header("Cache-Control", "no-cache")
      .body(())
      .map_err(|e| AppError::network_error(format!("Failed to build WebSocket request: {}", e)))?;
    
    let connection_result = connect_async(request).await;
    let (ws_stream, _) = match connection_result {
      Ok(connection) => {
        info!("✅ Successfully connected to TradingView WebSocket");
        connection
      }
      Err(e) => {
        warn!("❌ TradingView WebSocket connection failed: {}", e);
        return Err(AppError::network_error(format!("WebSocket connection failed: {}", e)));
      }
    };

    let (mut write, mut read) = ws_stream.split();
    
    // Process one symbol at a time for reliable extraction
    let mut all_eps_data = Vec::new();
    
    for symbol in &symbols {
      info!("📊 Processing symbol: {}", symbol);
      
      match self.extract_symbol_eps_data(&mut write, &mut read, symbol).await {
        Ok(eps_data) => {
          info!("✅ Successfully extracted EPS data for {}", symbol);
          all_eps_data.push(eps_data);
        }
        Err(e) => {
          warn!("⚠️ Failed to extract EPS data for {}: {}", symbol, e);
          // Continue processing other symbols instead of adding fallback data
          continue;
        }
      }
      
      // Small delay between symbols
      sleep(Duration::from_millis(500)).await;
    }
    
    info!("🎯 WebSocket extraction complete: {} symbols processed", all_eps_data.len());
    Ok(all_eps_data)
  }
  
  /// Extract EPS data using EXACT Node.js devtools flow
  async fn extract_symbol_eps_data(
    &mut self,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    symbol: &str
  ) -> Result<EPSWebSocketData, AppError> {
    
    info!("🔥 Starting TradingView WebSocket Extraction");
    
    // Reset state
    self.series_ready = false;
    self.symbol_resolved = false;
    self.extracted_eps_data = None;
    
    // Create chart session
    self.chart_session = Some(format!("cs_{}", self.generate_id(12)));
    let chart_session = self.chart_session.as_ref().unwrap();
    
    self.send_message(write, &json!({
      "m": "chart_create_session",
      "p": [chart_session, ""]
    })).await?;
    
    // Resolve symbol
    let symbol_id = "sds_sym_1";
    self.send_message(write, &json!({
      "m": "resolve_symbol",
      "p": [
        chart_session,
        symbol_id,
        format!("={{\"adjustment\":\"splits\",\"symbol\":\"{}\"}}", symbol)
      ]
    })).await?;
    
    // Create series
    let series_id = "sds_1";
    self.send_message(write, &json!({
      "m": "create_series",
      "p": [chart_session, series_id, "s1", symbol_id, "1D", 300, ""]
    })).await?;
    
    // Create quote session for additional data
    self.quote_session = Some(format!("qs_{}", self.generate_id(12)));
    let quote_session = self.quote_session.as_ref().unwrap();
    
    self.send_message(write, &json!({
      "m": "quote_create_session",
      "p": [quote_session]
    })).await?;
    
    self.send_message(write, &json!({
      "m": "quote_add_symbols",
      "p": [quote_session, format!("{}:{}", "NASDAQ", symbol)] // Try NASDAQ first
    })).await?;
    
    info!("⏳ Waiting for symbol resolution and series creation...");
    
    // Process messages and wait for series completion
    self.process_messages_until_complete(read, symbol, write).await
  }
  
  /// Process messages until EPS extraction is complete (exact Node.js logic)
  async fn process_messages_until_complete(
    &mut self,
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    symbol: &str,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>
  ) -> Result<EPSWebSocketData, AppError> {
    use tokio::time::{timeout, Duration};
    
    let extraction_timeout = timeout(Duration::from_millis(25000), async {
      while let Some(message) = read.next().await {
        if let Ok(Message::Text(text)) = message {
          let messages = self.parse_tradingview_message(&text);
          
          for parsed in messages {
            if let Some(method) = parsed["m"].as_str() {
              info!("📨 Received message type: {}", method);
              
              self.process_parsed_message(&parsed, symbol, write).await;
              
              // Check if we have extracted EPS data
              if let Some(eps_data) = &self.extracted_eps_data {
                info!("✅ EPS extraction completed!");
                return Ok(eps_data.clone());
              }
            }
          }
        }
      }
      Err(AppError::network_error("WebSocket closed before EPS extraction completed".to_string()))
    }).await;
    
    match extraction_timeout {
      Ok(Ok(eps_data)) => Ok(eps_data),
      Ok(Err(e)) => Err(e),
      Err(_) => {
        warn!("⏰ Extraction timeout - using any partial data");
        if let Some(eps_data) = &self.extracted_eps_data {
          Ok(eps_data.clone())
        } else {
          Err(AppError::network_error(format!("Timeout waiting for EPS data for {}", symbol)))
        }
      }
    }
  }
  
  /// Process parsed message (exact Node.js devtools logic)
  async fn process_parsed_message(
    &mut self,
    parsed: &Value,
    symbol: &str,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>
  ) {
    if let Some(method) = parsed["m"].as_str() {
      match method {
        "symbol_resolved" => {
          self.process_symbol_resolved_exact(parsed);
        }
        "qsd" | "quote_series_data" => {
          self.process_quote_data(parsed);
        }
        "timescale_update" | "tu" => {
          if let Some(eps_data) = self.process_timescale_update(parsed, symbol) {
            self.extracted_eps_data = Some(eps_data);
          }
        }
        "du" | "data_update" => {
          if let Some(eps_data) = self.process_data_update(parsed, symbol) {
            self.extracted_eps_data = Some(eps_data);
          }
        }
        "series_loading" => {
          info!("📊 Series loading...");
          self.series_ready = false;
        }
        "series_completed" | "series_loaded" => {
          self.process_series_completed(parsed, write).await;
        }
        "study_loading" => {
          if let Some(study_id) = parsed["p"].as_array().and_then(|p| p.get(0)).and_then(|s| s.as_str()) {
            info!("📈 Study loading: {}", study_id);
          }
        }
        "study_completed" | "study_loaded" => {
          if let Some(study_id) = parsed["p"].as_array().and_then(|p| p.get(0)).and_then(|s| s.as_str()) {
            info!("📈 Study completed: {}", study_id);
            if study_id.contains("st4") {
              info!("🎯 Earnings study (st4) completed - checking for EPS data");
            }
          }
        }
        "critical_error" | "protocol_error" => {
          self.handle_protocol_error(parsed);
        }
        _ => {
          if self.debug {
            debug!("❓ Unknown message type: {}", method);
          }
        }
      }
    }
  }
  
  
  
  /// Extract EPS data from st4 earnings study array (EXACT Node.js devtools logic)
  fn extract_eps_from_st4_exact(&self, st4_array: &[Value], symbol: &str) -> Vec<QuarterlyEPSData> {
    let mut quarterly_data: Vec<QuarterlyEPSData> = Vec::new();
    let current_timestamp = chrono::Utc::now().timestamp();

    // Log the ST4 data structure like Node.js devtools
    if self.debug && st4_array.len() >= 2 {
      let sample = &st4_array[..2.min(st4_array.len())];
      if let Ok(json_str) = serde_json::to_string_pretty(&sample) {
        info!("📊 ST4 data structure: {}", json_str);
      }
    }
    
    for (index, entry) in st4_array.iter().enumerate() {
      if let Some(v) = entry["v"].as_array() {
        let i_value = entry["i"].as_i64().unwrap_or(0);
        
        // Log ALL entries with ALL fields to find earnings_release_calendar_date
        if self.debug && index < 3 {  // Show first 3 entries in debug mode
          info!("📊 ST4 Entry {}: i={}, {} fields total", index, i_value, v.len());

          // Show ALL v[] indices to find future earnings field
          for (idx, val) in v.iter().enumerate() {
            let val_str = match val {
              Value::Number(n) => {
                // Check if this might be a future timestamp
                if let Some(ts) = n.as_i64() {
                  if ts > 1000000000 {  // Unix timestamp
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                      .or_else(|| chrono::DateTime::from_timestamp(ts / 1000, 0));
                    if let Some(date) = dt {
                      format!("{} ({})", n, date.format("%Y-%m-%d"))
                    } else {
                      n.to_string()
                    }
                  } else {
                    n.to_string()
                  }
                } else {
                  n.to_string()
                }
              },
              Value::String(s) => format!("\"{}\"", s),
              Value::Null => "null".to_string(),
              Value::Bool(b) => b.to_string(),
              _ => "?".to_string()
            };
            info!("      v[{}] = {}", idx, val_str);
          }
          info!("");
        }
        
        // Extract data from st4 format (EXACT Node.js logic):
        // v[0] = earnings timestamp (seconds), v[1] = estimated_eps, v[3] = quarter_end, v[4] = earnings_announcement (ms), v[5] = actual_eps
        let earnings_timestamp = v.get(0)
          .and_then(|v| v.as_f64())
          .map(|f| f as i64)
          .unwrap_or(0);
        let estimated_eps = v.get(1).and_then(|v| v.as_f64());
        let quarter_end_timestamp = v.get(3)
          .and_then(|v| v.as_f64())
          .map(|f| f as i64);
        let earnings_announcement_timestamp_ms = v.get(4)
          .and_then(|v| v.as_f64())
          .map(|f| f as i64);
        let actual_eps = v.get(5).and_then(|v| v.as_f64());

        // CRITICAL: Check if this is a FUTURE earnings entry (i_value > 100 indicates future/unreported)
        let is_future_earnings = i_value > 100;
        if is_future_earnings {
          info!("🎯 FOUND FUTURE EARNINGS: i={}, announcement timestamp: {:?}",
                i_value, earnings_announcement_timestamp_ms);
        }
        
        // Handle FUTURE earnings differently (they haven't been reported yet)
        let mut actual_eps_value = None;

        if is_future_earnings {
          // For future earnings, use estimated_eps as the actual value
          // v[5] will be 1e100 (placeholder for unreported)
          if let Some(eps) = estimated_eps {
            if eps < 1e50 && self.is_valid_quarterly_eps(eps) {  // Filter out 1e100 placeholder
              actual_eps_value = Some(eps);
              info!("📅 Future earnings estimated EPS: {}", eps);
            }
          }
        } else {
          // For historical earnings, use actual reported EPS
          // Try v[5] first (observed to be actual EPS)
          if let Some(eps) = actual_eps {
            if v.len() > 5 && self.is_valid_quarterly_eps(eps) {
              actual_eps_value = Some(eps);
            }
          }

          // Try v[1] as backup (might be estimated EPS)
          if actual_eps_value.is_none() {
            if let Some(eps) = estimated_eps {
              if v.len() > 1 && self.is_valid_quarterly_eps(eps) {
                actual_eps_value = Some(eps);
              }
            }
          }
        }

        // Extract earnings data (both historical and future)
        if let Some(eps_value) = actual_eps_value {
          let fiscal_period = self.timestamp_to_fiscal_period(earnings_timestamp);
          
          // Convert earnings announcement timestamp from milliseconds to seconds
          let estimated_earnings_date = earnings_announcement_timestamp_ms
            .map(|ms| ms / 1000)
            .unwrap_or(earnings_timestamp);
          
          // Generate quarter end date from quarter end timestamp
          let quarter_end_date = quarter_end_timestamp
            .map(|ts| {
              use chrono::{Utc, TimeZone};
              Utc.timestamp_opt(ts, 0)
                .single()
                .unwrap_or_default()
                .format("%Y-%m-%d")
                .to_string()
            });
          
          // Store estimated EPS if different from actual EPS
          let stored_estimated_eps = if let Some(est) = estimated_eps {
            if (est - eps_value).abs() > 0.01 { Some(est) } else { None }
          } else { None };
          
          // Log EPS values to diagnose frontend display issue
          info!("📊 WebSocket EPS data - Symbol: {}, Quarter: {}, EPS: {}", 
                symbol, fiscal_period, eps_value);
          
          // Extract fiscal quarter number from timestamp for meaningful quarter numbering
          let fiscal_quarter_number = self.extract_fiscal_quarter_number(earnings_timestamp);
          
          quarterly_data.push(QuarterlyEPSData {
            quarter_number: fiscal_quarter_number,
            period: fiscal_period.clone(),
            actual_eps: eps_value,
            timestamp: earnings_timestamp,
            estimated_eps: stored_estimated_eps,
            is_reported: !is_future_earnings,  // Future earnings are not yet reported
            beat_estimate: None,
            eps_type: if is_future_earnings { "st4_future_estimate".to_string() } else { "st4_earnings_study".to_string() },
            source: if is_future_earnings { "st4_future_calendar".to_string() } else { "st4_earnings_data".to_string() },
            quarter_end_date: quarter_end_date.clone(),
            estimated_earnings_date: earnings_announcement_timestamp_ms.map(|ms| ms / 1000),
            price_data: None,
            // Legacy fields for backward compatibility
            eps: eps_value,
            quarter_name: fiscal_period.clone(),
          });

          if is_future_earnings {
            info!("🎯 Extracted FUTURE EPS: {} = {} (estimated) [announcement: {}, quarter_end: {:?}]",
                  fiscal_period, eps_value, estimated_earnings_date, quarter_end_date);
          } else {
            info!("✅ Extracted EPS: {} = {} (from st4 v[5]) [announcement: {}, quarter_end: {:?}]",
                  fiscal_period, eps_value, estimated_earnings_date, quarter_end_date);
          }
        }
      }
    }
    
    if !quarterly_data.is_empty() {
      // Sort by timestamp (newest first) like Node.js
      quarterly_data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
      
      // Add price correlation like Node.js (if we have price data)
      quarterly_data = self.correlate_price_with_earnings(quarterly_data);
    } else {
      warn!("⚠️ No EPS data extracted from st4 for {}", symbol);
    }
    
    quarterly_data
  }
  
  /// Process timescale_update messages (exact Node.js devtools logic)
  fn process_timescale_update(&mut self, parsed: &Value, symbol: &str) -> Option<EPSWebSocketData> {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        let session_id = params[0].as_str().unwrap_or("");
        let data = &params[1];
        
        info!("📈 Timescale update for: {}", session_id);
        if self.debug {
          let data_str = serde_json::to_string(data).unwrap_or_default();
          info!("Data structure: {}", &data_str[..data_str.len().min(200)]);
        }
        
        // Extract st4 earnings data (exact match to Node.js)
        if let Some(st4_obj) = data.get("st4") {
          if let Some(st4_data) = st4_obj.get("st").and_then(|v| v.as_array()) {
            info!("🎯 Found st4 earnings data with {} entries for {}!", st4_data.len(), symbol);
            info!("🔥 EXTRACTING EPS FROM ST4 EARNINGS STUDY: st4_earnings_data");
            
            let quarterly_data = self.extract_eps_from_st4_exact(st4_data, symbol);
            
            if !quarterly_data.is_empty() {
              info!("✅ Extracted {} EPS values from st4!", quarterly_data.len());
              return Some(self.build_eps_websocket_data(symbol, quarterly_data));
            }
          }
        }
        
        // Extract price data from series (sds_1)
        if let Some(series_obj) = data.get("sds_1") {
          if let Some(series_data) = series_obj.get("s").and_then(|v| v.as_array()) {
            info!("📊 Found series data with {} candles", series_data.len());
            self.extract_price_from_timescale_update(series_data);
          }
        }
      }
    }
    None
  }
  
  /// Process data update (du) messages (exact Node.js devtools logic)
  fn process_data_update(&mut self, parsed: &Value, symbol: &str) -> Option<EPSWebSocketData> {
    info!("📊 Processing data update");
    
    if self.debug {
      if let Ok(json_str) = serde_json::to_string(&parsed) {
        let preview = if json_str.len() > 500 { &json_str[..500] } else { &json_str };
        info!("🔍 Full DU message structure: {}", preview);
      }
    }
    
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        let study_id = params[0].as_str().unwrap_or("");
        let data_obj = &params[1];
        
        info!("📈 Data update for study: {}", study_id);
        
        // Check if this is an earnings study update (st4)
        if study_id.contains("st4") {
          info!("🎯 Found st4 (Earnings) data update!");
          
          // Try multiple paths to find EPS data like Node.js
          if let Some(st_obj) = data_obj.as_object() {
            // Check direct st property
            if let Some(st_data) = st_obj.get("st").and_then(|v| v.as_array()) {
              info!("📊 Found st array with {} items", st_data.len());
              let quarterly_data = self.extract_eps_from_st4_exact(st_data, symbol);
              if !quarterly_data.is_empty() {
                return Some(self.build_eps_websocket_data(symbol, quarterly_data));
              }
            }
            
            // Check nested st4 property
            if let Some(st4_obj) = st_obj.get("st4") {
              info!("📊 Found st4 property");
              if let Some(st_data) = st4_obj.get("st").and_then(|v| v.as_array()) {
                let quarterly_data = self.extract_eps_from_st4_exact(st_data, symbol);
                if !quarterly_data.is_empty() {
                  return Some(self.build_eps_websocket_data(symbol, quarterly_data));
                }
              }
            }
          }
        }
      }
    }
    None
  }
  
  /// Process symbol resolution (exact Node.js logic)
  fn process_symbol_resolved_exact(&mut self, parsed: &Value) {
    info!("🔍 Processing symbol resolution");
    self.symbol_resolved = true;
    
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 3 {
        if let Some(symbol_data) = params[2].as_object() {
          if let Some(exchange) = symbol_data.get("exchange").and_then(|v| v.as_str()) {
            info!("✅ Detected exchange: {}", exchange);
          }
          if let Some(description) = symbol_data.get("description").and_then(|v| v.as_str()) {
            info!("🏢 Company: {}", description);
          }
          if let Some(symbol_type) = symbol_data.get("type").and_then(|v| v.as_str()) {
            info!("📊 Type: {}", symbol_type);
          }
        }
      }
    }
  }
  
  /// Process series completion and create studies (exact Node.js logic)
  async fn process_series_completed(
    &mut self,
    parsed: &Value,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>
  ) {
    info!("📊 Series data completed");
    self.series_ready = true;
    
    // Extract price data from series completion if available
    if let Some(params) = parsed["p"].as_array() {
      self.extract_price_data_from_series(params);
    }
    
    // Now that series is ready, create studies
    self.create_studies_after_series(write).await;
  }
  
  /// Create studies after series is ready (exact Node.js logic)
  async fn create_studies_after_series(
    &mut self,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>
  ) {
    if !self.series_ready {
      return;
    }
    
    info!("📈 Series ready, creating studies...");
    
    let chart_session = self.chart_session.as_ref().unwrap();
    let series_id = "sds_1";
    
    // Create studies exactly like Node.js
    let studies = vec![
      ("st4", "Earnings@tv-basicstudies-255"),
      ("st2", "Dividends@tv-basicstudies-255"),
      ("st3", "Splits@tv-basicstudies-255")
    ];
    
    for (study_id, study_name) in studies {
      let _ = self.send_message(write, &json!({
        "m": "create_study",
        "p": [chart_session, study_id, "st1", series_id, study_name, {}]
      })).await;
      
      sleep(Duration::from_millis(50)).await;
    }
    
    info!("✅ Studies created, waiting for data...");
  }
  
  /// Extract price data from series response
  fn extract_price_data_from_series(&mut self, series_data: &[Value]) {
    if series_data.len() > 1 {
      if let Some(price_info) = series_data[1].as_object() {
        if let Some(current_price) = price_info.get("lp").and_then(|v| v.as_f64()) {
          info!("💲 Current price from series: {}", current_price);
        }
      }
    }
  }
  
  /// Process quote data (extract financial context)
  fn process_quote_data(&mut self, parsed: &Value) {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        if let Some(status) = params[1].get("s").and_then(|s| s.as_str()) {
          if status == "ok" {
            if let Some(data) = params[1].get("v").and_then(|v| v.as_object()) {
              // Extract financial context like Node.js
              if let Some(current_price) = data.get("lp").and_then(|v| v.as_f64()) {
                info!("💰 Quote price: {}", current_price);
                
                // CRITICAL FIX: Store the quote price in price_data instead of just logging
                let current_timestamp = chrono::Utc::now().timestamp();
                let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
                
                // Create PriceData entry with the real quote price
                let price_data = PriceData {
                  timestamp: current_timestamp,
                  date: current_date,
                  open: current_price,      // Use current price as all OHLC since it's real-time quote
                  high: current_price,
                  low: current_price, 
                  close: current_price,     // This is the key field used in price extraction
                  volume: data.get("volume").and_then(|v| v.as_i64()).unwrap_or(0),
                };
                
                // Store in price_data collection so it can be accessed later
                self.price_data.insert(current_timestamp, price_data);
                info!("✅ Stored real quote price {} in price_data collection", current_price);
              }
              if let Some(market_cap) = data.get("market_cap_basic").and_then(|v| v.as_f64()) {
                info!("📈 Market cap: {}", market_cap);
              }
            }
          }
        }
      }
    }
  }
  
  /// Handle protocol errors like Node.js
  fn handle_protocol_error(&self, parsed: &Value) {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        let session_id = params[0].as_str().unwrap_or("unknown");
        let error_type = params[1].as_str().unwrap_or("unknown");
        warn!("❌ Protocol error in {}: {}", session_id, error_type);
        
        if params.len() >= 3 {
          if let Some(details) = params[2].as_str() {
            warn!("   Details: {}", details);
          }
        }
      }
    }
  }
  
  /// Extract price data from timescale updates (ported from Node.js devtools)
  fn extract_price_from_timescale_update(&mut self, data: &[Value]) {
    for candle in data {
      if let Some(v) = candle["v"].as_array() {
        if v.len() >= 6 {
          if let (Some(timestamp), Some(open), Some(high), Some(low), Some(close), Some(volume)) = (
            v[0].as_i64(),
            v[1].as_f64(),
            v[2].as_f64(), 
            v[3].as_f64(),
            v[4].as_f64(),
            v[5].as_i64()
          ) {
            let date = chrono::Utc.timestamp_opt(timestamp, 0)
              .single()
              .unwrap_or_default()
              .format("%Y-%m-%d")
              .to_string();
            
            let price_data = PriceData {
              timestamp,
              date,
              open,
              high,
              low,
              close,
              volume,
            };
            
            self.price_data.insert(timestamp, price_data);
          }
        }
      }
    }
    
    if !self.price_data.is_empty() {
      debug!("📊 Extracted {} price points from timescale", self.price_data.len());
    }
  }
  
  /// Build complete EPS WebSocket data structure
  fn build_eps_websocket_data(&self, symbol: &str, quarterly_data: Vec<QuarterlyEPSData>) -> EPSWebSocketData {
    let current_eps = quarterly_data.first().map(|q| q.eps).unwrap_or(0.0);
    let price_data: Vec<PriceData> = self.price_data.values().cloned().collect();
    let current_price = price_data.last().map(|p| p.close).unwrap_or(0.0);
    let current_volume = price_data.last().map(|p| p.volume).unwrap_or(0) as f64;
    
    EPSWebSocketData {
      symbol: symbol.to_string(),
      current_eps,
      quarterly_eps: current_eps,
      historical_eps: quarterly_data.iter().map(|q| q.eps).collect(),
      quarterly_data,
      price_data,
      earnings_per_share_basic_ttm: current_eps, // Remove TTM conversion - use quarterly EPS directly
      market_cap_basic: 0.0,
      price_current: current_price,
      volume: current_volume,
      sector: "Technology".to_string(),
      country: "US".to_string(),
      company_name: symbol.to_string(),
    }
  }
  
  /// Convert timestamp to fiscal period (exact Node.js devtools format)
  fn timestamp_to_fiscal_period(&self, timestamp: i64) -> String {
    use chrono::{Utc, TimeZone};
    
    if timestamp == 0 {
      return "Unknown".to_string();
    }
    
    let dt = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_default();
    let year = dt.year();
    let month = dt.month();
    
    // Map to fiscal quarters like Node.js devtools
    let (quarter, fiscal_year) = match month {
      1..=3 => (1, year),     // Q1 
      4..=6 => (2, year),     // Q2
      7..=9 => (3, year),     // Q3 
      10..=12 => (4, year),   // Q4
      _ => (1, year),
    };
    
    format!("{}-Q{}", fiscal_year, quarter)
  }
  
  /// Extract fiscal quarter number (1-4) from timestamp for meaningful quarter identification
  fn extract_fiscal_quarter_number(&self, timestamp: i64) -> usize {
    use chrono::{Utc, TimeZone};
    
    let dt = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_default();
    let month = dt.month();
    
    // Map to fiscal quarters (same logic as timestamp_to_fiscal_period)
    match month {
      1..=3 => 1,     // Q1 
      4..=6 => 2,     // Q2
      7..=9 => 3,     // Q3 
      10..=12 => 4,   // Q4
      _ => 1,
    }
  }
  
  /// Parse TradingView message format (ported from Node.js devtools)
  fn parse_tradingview_message(&self, text: &str) -> Vec<Value> {
    let mut messages = Vec::new();
    
    // Handle heartbeat messages
    if text.starts_with("~h~") {
      return messages;
    }
    
    if !text.starts_with("~m~") {
      return messages;
    }
    
    let mut remaining = text;
    while remaining.starts_with("~m~") {
      // Find the length
      let length_start = 3;
      if let Some(length_end) = remaining[length_start..].find("~m~") {
        let length_str = &remaining[length_start..length_start + length_end];
        if let Ok(message_length) = length_str.parse::<usize>() {
          let message_start = length_start + length_end + 3;
          if message_start + message_length <= remaining.len() {
            let json_str = &remaining[message_start..message_start + message_length];
            
            if let Ok(parsed) = serde_json::from_str::<Value>(json_str) {
              messages.push(parsed);
            }
            
            remaining = &remaining[message_start + message_length..];
          } else {
            break;
          }
        } else {
          break;
        }
      } else {
        break;
      }
    }
    
    messages
  }

  /// Dynamic validation for quarterly EPS values - no hardcoded limits
  fn is_valid_quarterly_eps(&self, eps: f64) -> bool {
    // Basic sanity checks
    if !eps.is_finite() || eps <= 0.0 {
      return false;
    }

    // Allow wide range for different markets and currencies
    // US stocks: typically 0.01 to 50.0 USD per share
    // International stocks: can be much higher (e.g., Taiwan stocks in TWD)
    // Accept any reasonable positive value up to 50,000 to handle all currencies
    if eps > 50000.0 {
      warn!("EPS value {} seems extremely high, might be an error", eps);
      return false;
    }

    // Accept very small values too (penny stocks, recent IPOs)
    if eps < 0.001 {
      warn!("EPS value {} is very small, might be noise", eps);
      return false;
    }

    // All other values are considered valid
    true
  }
  
  /// Format message for TradingView WebSocket (ported from Node.js devtools)
  fn format_tradingview_message(&self, msg: &Value) -> String {
    let json_str = serde_json::to_string(msg).unwrap_or_default();
    format!("~m~{}~m~{}", json_str.len(), json_str)
  }
  
  /// Send message to TradingView WebSocket (exact Node.js format)
  async fn send_message(
    &self,
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    msg: &Value
  ) -> Result<(), AppError> {
    let formatted = self.format_tradingview_message(msg);
    
    write.send(Message::Text(formatted)).await.map_err(|e| {
      AppError::network_error(format!("Failed to send WebSocket message: {}", e))
    })?;
    
    if let Some(method) = msg["m"].as_str() {
      info!("➡️ Sent: {}", method);
    }
    
    Ok(())
  }
  
  /// Generate unique ID for TradingView messages (exact Node.js format)
  fn generate_id(&self, length: usize) -> String {
    Uuid::new_v4()
      .to_string()
      .replace("-", "")
      .chars()
      .take(length)
      .collect()
  }
  
  
  /// Convert WebSocket data to frontend EPS format with price correlation
  pub fn convert_to_frontend_format(&self, websocket_data: Vec<EPSWebSocketData>) -> Vec<FrontendEPSData> {
    websocket_data.into_iter().map(|ws_data| {
      // Calculate QoQ growth from quarterly data
      let qoq_growth = if ws_data.quarterly_data.len() >= 2 {
        let current = ws_data.quarterly_data[0].eps;
        let previous = ws_data.quarterly_data[1].eps;
        if previous != 0.0 {
          ((current - previous) / previous) * 100.0
        } else {
          0.0
        }
      } else {
        0.0
      };

      // Enhanced ranking score using price correlation and volatility
      let price_factor = if !ws_data.price_data.is_empty() {
        let price_volatility = self.calculate_price_volatility(&ws_data.price_data);
        let eps_price_correlation = self.calculate_eps_price_correlation(&ws_data.quarterly_data, &ws_data.price_data);
        price_volatility * 0.1 + eps_price_correlation * 0.2
      } else {
        0.0
      };
      
      let ranking_score = ws_data.current_eps * 0.4 + qoq_growth * 0.3 + price_factor + (ws_data.market_cap_basic / 1_000_000_000.0) * 0.1;

      FrontendEPSData {
        id: format!("{}-{}", ws_data.symbol, uuid::Uuid::new_v4().to_string()[..8].to_string()),
        symbol: ws_data.symbol.clone(),
        name: ws_data.company_name.clone(),
        company_name: ws_data.company_name,
        current_eps: ws_data.current_eps,
        previous_eps: ws_data.historical_eps.get(1).copied().unwrap_or(ws_data.current_eps * 0.9),
        growth_rate: qoq_growth,
        qoq_growth,
        market_cap: ws_data.market_cap_basic as i64,
        price_current: ws_data.price_current,
        volume: ws_data.volume as i64,
        country: ws_data.country,
        sector: ws_data.sector,
        ranking_score,
        last_updated: chrono::Utc::now(),
      }
    }).collect()
  }
  
  /// Calculate price volatility from price data
  fn calculate_price_volatility(&self, price_data: &[PriceData]) -> f64 {
    if price_data.len() < 2 {
      return 0.0;
    }
    
    let prices: Vec<f64> = price_data.iter().map(|p| p.close).collect();
    let mean = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;
    variance.sqrt() / mean
  }
  
  /// Calculate correlation between EPS and price movements
  fn calculate_eps_price_correlation(&self, eps_data: &[QuarterlyEPSData], price_data: &[PriceData]) -> f64 {
    if eps_data.len() < 2 || price_data.is_empty() {
      return 0.0;
    }
    
    // Find price movements around EPS announcement dates
    let mut correlation_score = 0.0;
    let mut valid_correlations = 0;
    
    for eps in eps_data {
      if let Some(price_before) = self.find_price_near_date(price_data, eps.timestamp - 86400) {
        if let Some(price_after) = self.find_price_near_date(price_data, eps.timestamp + 86400) {
          let price_change_pct = ((price_after.close - price_before.close) / price_before.close) * 100.0;
          
          // Positive EPS should correlate with positive price movement
          if (eps.eps > 0.0 && price_change_pct > 0.0) || (eps.eps < 0.0 && price_change_pct < 0.0) {
            correlation_score += 1.0;
          }
          valid_correlations += 1;
        }
      }
    }
    
    if valid_correlations > 0 {
      correlation_score / valid_correlations as f64
    } else {
      0.0
    }
  }
  
  /// Find price data near a specific timestamp
  fn find_price_near_date<'a>(&self, price_data: &'a [PriceData], target_timestamp: i64) -> Option<&'a PriceData> {
    price_data.iter()
      .min_by_key(|p| (p.timestamp - target_timestamp).abs())
  }
  
  /// Create comprehensive EPS-Price correlation (EXACT Node.js logic)
  fn correlate_price_with_earnings(&self, quarterly_data: Vec<QuarterlyEPSData>) -> Vec<QuarterlyEPSData> {
    info!("🔄 Creating comprehensive EPS-Price correlation");
    
    quarterly_data.into_iter().map(|mut quarter| {
      // Use VWAP-based volatility-aware price correlation
      let price_impact = if let Some(earnings_date) = quarter.estimated_earnings_date {
        self.find_nearest_price_data_vwap(earnings_date, &quarter.period)
      } else {
        self.find_nearest_price_data_vwap(quarter.timestamp, &quarter.period)
      };
      
      quarter.price_data = price_impact;
      quarter
    }).collect()
  }
  
  /// Enhanced volatility-aware price correlation using VWAP
  fn find_nearest_price_data_vwap(&self, earnings_timestamp: i64, quarter_period: &str) -> Option<PriceImpactData> {
    if self.price_data.is_empty() {
      warn!("⚠️ No price data available for correlation");
      return None;
    }
    
    info!("🔍 VWAP-based price correlation for {} earnings (timestamp: {})", quarter_period, earnings_timestamp);
    
    let one_day = 86400;
    
    // Calculate volatility using ATR
    let atr = self.calculate_atr(earnings_timestamp, 14);
    let avg_price = self.price_data.values()
      .filter(|p| p.close > 0.0)
      .map(|p| p.close)
      .sum::<f64>() / self.price_data.len() as f64;
    
    let volatility_ratio = if avg_price > 0.0 { atr / avg_price } else { 0.0 };
    
    // Dynamic correlation window based on volatility
    let (primary_window, fallback_window) = if volatility_ratio > 0.03 { // High volatility (>3%)
      info!("📈 High volatility detected ({:.2}%), using wider correlation window", volatility_ratio * 100.0);
      (7, 14) // Wider window for volatile stocks
    } else {
      info!("📊 Normal volatility ({:.2}%), using standard correlation window", volatility_ratio * 100.0);
      (3, 7)  // Narrower window for stable stocks
    };
    
    // Calculate VWAP for periods before/after earnings
    let vwap_before = self.calculate_vwap(
      earnings_timestamp - (primary_window as i64 * one_day),
      earnings_timestamp - one_day
    );
    
    let vwap_after = self.calculate_vwap(
      earnings_timestamp + one_day,
      earnings_timestamp + (primary_window as i64 * one_day)
    );
    
    // Fallback to individual day prices if VWAP fails
    let (before_price, days_before) = if let Some(vwap) = vwap_before {
      (vwap, primary_window / 2) // Use middle of VWAP period for days calculation
    } else {
      // Fallback to closest single day price
      let mut best_before: Option<(f64, i32)> = None;
      for step in 1..=fallback_window {
        let before_timestamp = earnings_timestamp - (step * one_day);
        if let Some(price_data) = self.price_data.get(&before_timestamp) {
          // Use stability-adjusted price
          let stability = self.calculate_price_stability(before_timestamp);
          let adjusted_price = if stability < 0.95 { // <95% stability, use typical price
            (price_data.high + price_data.low + price_data.close) / 3.0
          } else {
            price_data.close
          };
          best_before = Some((adjusted_price, step as i32));
          break;
        }
      }
      best_before.unwrap_or((0.0, 0))
    };
    
    let (after_price, days_after) = if let Some(vwap) = vwap_after {
      (vwap, primary_window / 2)
    } else {
      // Fallback to closest single day price
      let mut best_after: Option<(f64, i32)> = None;
      for step in 1..=fallback_window {
        let after_timestamp = earnings_timestamp + (step * one_day);
        if let Some(price_data) = self.price_data.get(&after_timestamp) {
          let stability = self.calculate_price_stability(after_timestamp);
          let adjusted_price = if stability < 0.95 {
            (price_data.high + price_data.low + price_data.close) / 3.0
          } else {
            price_data.close
          };
          best_after = Some((adjusted_price, step as i32));
          break;
        }
      }
      best_after.unwrap_or((0.0, 0))
    };
    
    // Calculate price impact with enhanced quality grading
    if before_price > 0.0 && after_price > 0.0 {
      let price_change = after_price - before_price;
      let percent_change = (price_change / before_price) * 100.0;
      
      // Enhanced quality grading including volatility factors
      let data_quality = if vwap_before.is_some() && vwap_after.is_some() {
        if volatility_ratio < 0.02 { "excellent_vwap" } else { "good_vwap" }
      } else if days_before <= 2 && days_after <= 2 && volatility_ratio < 0.03 {
        "excellent"
      } else if days_before <= primary_window && days_after <= primary_window {
        if volatility_ratio > 0.05 { "fair_volatile" } else { "good" }
      } else {
        "fair"
      };
      
      info!("💹 VWAP Price impact for {}: {:.2}% ({}) [{}] - volatility: {:.2}%", 
            quarter_period, percent_change, 
            if percent_change > 0.0 { "positive" } else { "negative" },
            data_quality, volatility_ratio * 100.0);
      
      return Some(PriceImpactData {
        pre_earnings_price: before_price,
        post_earnings_price: after_price,
        price_change: (price_change * 100.0).round() / 100.0,
        percent_change: (percent_change * 100.0).round() / 100.0,
        earnings_impact: if percent_change > 0.0 { "positive".to_string() } else { "negative".to_string() },
        days_before: days_before as i32,
        days_after: days_after as i32,
        volume_before: 0, // VWAP aggregates volume
        volume_after: 0,
        volume_change: "vwap_calculated".to_string(),
        data_quality: data_quality.to_string(),
      });
    }
    
    warn!("⚠️ Insufficient price data for VWAP correlation for {}", quarter_period);
    None
  }

  /// Calculate Volume-Weighted Average Price (VWAP) over multiple days to reduce variance
  fn calculate_vwap(&self, start_timestamp: i64, end_timestamp: i64) -> Option<f64> {
    let mut total_pv = 0.0; // price * volume
    let mut total_volume = 0.0;
    let mut data_count = 0;
    
    // Collect all price data within the time range
    for (timestamp, price_data) in &self.price_data {
      if *timestamp >= start_timestamp && *timestamp <= end_timestamp {
        // Use typical price (H+L+C)/3 for VWAP calculation
        let typical_price = (price_data.high + price_data.low + price_data.close) / 3.0;
        let volume = price_data.volume as f64;
        
        if volume > 0.0 { // Only include days with volume
          total_pv += typical_price * volume;
          total_volume += volume;
          data_count += 1;
        }
      }
    }
    
    if total_volume > 0.0 && data_count > 0 {
      let vwap = total_pv / total_volume;
      info!("📊 VWAP calculated: ${:.2} over {} days (volume: {:.0})", 
            vwap, data_count, total_volume);
      Some(vwap)
    } else {
      None
    }
  }
  
  /// Calculate Average True Range (ATR) for volatility measurement
  fn calculate_atr(&self, timestamp: i64, periods: i32) -> f64 {
    let one_day = 86400;
    let mut true_ranges: Vec<f64> = Vec::new();
    
    // Collect price data around the timestamp
    let mut sorted_data: Vec<_> = self.price_data
      .iter()
      .filter(|(ts, _)| (**ts >= timestamp - (periods as i64 * one_day)) && (**ts <= timestamp + (periods as i64 * one_day)))
      .collect();
    
    sorted_data.sort_by_key(|(ts, _)| *ts);
    
    for i in 1..sorted_data.len().min(periods as usize) {
      let current = sorted_data[i].1;
      let previous = sorted_data[i-1].1;
      
      // True Range = max(H-L, |H-Cp|, |L-Cp|)
      let hl = current.high - current.low;
      let hcp = (current.high - previous.close).abs();
      let lcp = (current.low - previous.close).abs();
      
      let tr = hl.max(hcp).max(lcp);
      if tr > 0.0 {
        true_ranges.push(tr);
      }
    }
    
    if !true_ranges.is_empty() {
      let atr = true_ranges.iter().sum::<f64>() / true_ranges.len() as f64;
      info!("📈 ATR calculated: {:.2} over {} periods", atr, true_ranges.len());
      atr
    } else {
      0.0
    }
  }
  
  /// Calculate price stability ratio using daily range
  fn calculate_price_stability(&self, timestamp: i64) -> f64 {
    if let Some(price_data) = self.price_data.get(&timestamp) {
      if price_data.close > 0.0 {
        let daily_range = (price_data.high - price_data.low) / price_data.close;
        info!("📊 Price stability for timestamp {}: {:.3} ({:.1}% daily range)", 
              timestamp, 1.0 - daily_range, daily_range * 100.0);
        return 1.0 - daily_range; // Higher value = more stable
      }
    }
    0.5 // Default middle stability
  }

}