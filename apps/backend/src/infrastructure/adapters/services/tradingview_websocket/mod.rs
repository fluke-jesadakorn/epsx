// TradingView WebSocket Service
// Modular WebSocket service for real-time EPS data extraction

pub mod types;
pub mod connection;
pub mod extractor;

pub use types::*;
pub use connection::*;
pub use extractor::*;

use std::collections::HashMap;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::StreamExt;
use serde_json::{ json, Value };
use tracing::{ debug, info, warn };

use epsx_contracts::errors::AppError;

const DEFAULT_WS_URL: &str = "wss://data.tradingview.com/socket.io/websocket";
const DEFAULT_ORIGIN: &str = "https://www.tradingview.com";

/// TradingView WebSocket service coordinator
pub struct TradingViewWebSocketService {
  symbols: Vec<String>,
  price_data: HashMap<i64, PriceData>,
  debug: bool,
  chart_session: Option<String>,
  quote_session: Option<String>,
  series_ready: bool,
  symbol_resolved: bool,
  extracted_eps_data: Option<EPSWebSocketData>,
  websocket_url: String,
  origin_url: String,
}

impl Default for TradingViewWebSocketService {
  fn default() -> Self {
    Self::new()
  }
}

impl TradingViewWebSocketService {
  pub fn new() -> Self {
    Self {
      symbols: Vec::new(),
      price_data: HashMap::new(),
      debug: true,
      chart_session: None,
      quote_session: None,
      series_ready: false,
      symbol_resolved: false,
      extracted_eps_data: None,
      websocket_url: DEFAULT_WS_URL.to_string(),
      origin_url: DEFAULT_ORIGIN.to_string(),
    }
  }

  /// Connect and fetch EPS data for multiple symbols
  pub async fn connect_and_fetch_eps_data(
    &mut self,
    symbols: Vec<String>
  ) -> Result<Vec<EPSWebSocketData>, AppError> {
    info!(
      "Starting TradingView WebSocket connection for {} symbols",
      symbols.len()
    );
    self.symbols = symbols.clone();

    let ws_stream = connection::connect_websocket(&self.websocket_url, &self.origin_url).await?;
    let (mut write, mut read) = ws_stream.split();

    let mut all_eps_data = Vec::new();

    for symbol in &symbols {
      info!("Processing symbol: {}", symbol);

      match self.extract_symbol_eps_data(&mut write, &mut read, symbol).await {
        Ok(eps_data) => {
          info!("Successfully extracted EPS data for {}", symbol);
          all_eps_data.push(eps_data);
        }
        Err(e) => {
          warn!("Failed to extract EPS data for {}: {}", symbol, e);
          continue;
        }
      }
    }

    info!(
      "TradingView WebSocket extraction complete: {}/{} symbols",
      all_eps_data.len(),
      symbols.len()
    );
    Ok(all_eps_data)
  }

  async fn extract_symbol_eps_data(
    &mut self,
    write: &mut futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
      Message
    >,
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    symbol: &str
  ) -> Result<EPSWebSocketData, AppError> {
    info!("Starting TradingView WebSocket Extraction");

    self.series_ready = false;
    self.symbol_resolved = false;
    self.extracted_eps_data = None;

    self.chart_session = Some(format!("cs_{}", connection::generate_id(12)));
    let chart_session = self.chart_session.as_ref().unwrap();

    connection::send_message(
      write,
      &json!({
      "m": "chart_create_session",
      "p": [chart_session, ""]
    })
    ).await?;

    let symbol_id = "sds_sym_1";
    connection::send_message(
      write,
      &json!({
      "m": "resolve_symbol",
      "p": [
        chart_session,
        symbol_id,
        format!("={{\"adjustment\":\"splits\",\"symbol\":\"{}\"}}", symbol)
      ]
    })
    ).await?;

    let series_id = "sds_1";
    connection::send_message(
      write,
      &json!({
      "m": "create_series",
      "p": [chart_session, series_id, "s1", symbol_id, "1D", 300, ""]
    })
    ).await?;

    self.quote_session = Some(format!("qs_{}", connection::generate_id(12)));
    let quote_session = self.quote_session.as_ref().unwrap();

    connection::send_message(
      write,
      &json!({
      "m": "quote_create_session",
      "p": [quote_session]
    })
    ).await?;

    connection::send_message(
      write,
      &json!({
      "m": "quote_add_symbols",
      "p": [quote_session, format!("{}:{}", "NASDAQ", symbol)]
    })
    ).await?;

    info!("Waiting for symbol resolution and series creation...");

    self.process_messages_until_complete(read, symbol, write).await
  }

  async fn process_messages_until_complete(
    &mut self,
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    symbol: &str,
    write: &mut futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
      Message
    >
  ) -> Result<EPSWebSocketData, AppError> {
    use tokio::time::timeout;

    let extraction_timeout = timeout(Duration::from_millis(25000), async {
      while let Some(message) = read.next().await {
        if let Ok(Message::Text(text)) = message {
          let messages = connection::parse_tradingview_message(&text);

          for parsed in messages {
            if let Some(method) = parsed["m"].as_str() {
              info!("Received message type: {}", method);

              self.process_parsed_message(&parsed, symbol, write).await;

              if self.extracted_eps_data.is_some() {
                return Ok(self.extracted_eps_data.clone().unwrap());
              }
            }
          }
        }
      }

      Err(
        AppError::internal_server_error(
          "No EPS data extracted before WebSocket closed"
        )
      )
    }).await;

    match extraction_timeout {
      Ok(result) => result,
      Err(_) => {
        warn!("WebSocket extraction timed out for {}", symbol);
        Err(
          AppError::internal_server_error(
            format!("WebSocket extraction timeout for {}", symbol)
          )
        )
      }
    }
  }

  async fn process_parsed_message(
    &mut self,
    parsed: &Value,
    symbol: &str,
    write: &mut futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
      Message
    >
  ) {
    let method = parsed["m"].as_str().unwrap_or("");

    match method {
      "symbol_resolved" => {
        self.symbol_resolved = true;
        info!("Symbol {} resolved", symbol);
      }
      "series_completed" => {
        if !self.series_ready {
          self.series_ready = true;
          info!("Series creation completed for {}", symbol);
          let _ = self.create_studies_after_series(write, symbol).await;
        }
      }
      "timescale_update" => {
        if let Some(eps_data) = self.process_timescale_update(parsed, symbol) {
          self.extracted_eps_data = Some(eps_data);
        }
      }
      "du" => {
        if let Some(eps_data) = self.process_data_update(parsed, symbol) {
          self.extracted_eps_data = Some(eps_data);
        }
      }
      "qsd" => {
        self.process_quote_data(parsed);
      }
      "critical_error" | "protocol_error" => {
        self.handle_protocol_error(parsed);
      }
      _ => {
        debug!("Unhandled message type: {}", method);
      }
    }
  }

  async fn create_studies_after_series(
    &mut self,
    write: &mut futures_util::stream::SplitSink<
      tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
      Message
    >,
    _symbol: &str
  ) -> Result<(), AppError> {
    let chart_session = self.chart_session.as_ref().unwrap();

    connection::send_message(
      write,
      &json!({
      "m": "create_study",
      "p": [
        chart_session,
        "st4",
        "st1",
        "sds_1",
        "Earnings@tv-basicstudies-246",
        json!({})
      ]
    })
    ).await?;

    info!("Created ST4 earnings study");
    Ok(())
  }

  fn process_timescale_update(
    &mut self,
    parsed: &Value,
    symbol: &str
  ) -> Option<EPSWebSocketData> {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        if let Some(st4_array) = params[1]["st4"].as_array() {
          info!(
            "Found ST4 earnings data in timescale_update: {} quarters",
            st4_array.len()
          );

          let quarterly_data = extractor::extract_eps_from_st4(
            st4_array,
            symbol,
            self.debug,
            |data| self.correlate_price_with_earnings(data)
          );

          return Some(
            extractor::build_eps_websocket_data(
              symbol,
              quarterly_data,
              self.price_data.values().cloned().collect(),
              "Unknown".to_string(),
              "Unknown".to_string(),
              symbol.to_string()
            )
          );
        }
      }
    }
    None
  }

  fn process_data_update(
    &mut self,
    parsed: &Value,
    symbol: &str
  ) -> Option<EPSWebSocketData> {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        if let Some(st4_data) = params[1]["st4"].as_object() {
          if let Some(st_array) = st4_data["st"].as_array() {
            info!(
              "Found ST4 earnings data in data_update: {} quarters",
              st_array.len()
            );

            let quarterly_data = extractor::extract_eps_from_st4(
              st_array,
              symbol,
              self.debug,
              |data| self.correlate_price_with_earnings(data)
            );

            return Some(
              extractor::build_eps_websocket_data(
                symbol,
                quarterly_data,
                self.price_data.values().cloned().collect(),
                "Unknown".to_string(),
                "Unknown".to_string(),
                symbol.to_string()
              )
            );
          }
        }
      }
    }
    None
  }

  fn process_quote_data(&mut self, parsed: &Value) {
    if let Some(params) = parsed["p"].as_array() {
      if params.len() >= 2 {
        if let Some(quote_data) = params[1].as_object() {
          debug!("Quote data received: {} fields", quote_data.len());
        }
      }
    }
  }

  fn handle_protocol_error(&self, parsed: &Value) {
    if let Some(params) = parsed["p"].as_array() {
      warn!("Protocol error: {:?}", params);
    }
  }

  fn correlate_price_with_earnings(
    &self,
    quarterly_data: Vec<QuarterlyEPSData>
  ) -> Vec<QuarterlyEPSData> {
    quarterly_data
  }

  /// Convert to frontend format
  pub fn convert_to_frontend_format(
    &self,
    websocket_data: Vec<EPSWebSocketData>
  ) -> Vec<FrontendEPSData> {
    websocket_data
      .into_iter()
      .map(|data| {
        let qoq_growth = if data.quarterly_data.len() >= 2 {
          let current = data.quarterly_data[0].actual_eps;
          let previous = data.quarterly_data[1].actual_eps;
          if previous != 0.0 {
            ((current - previous) / previous) * 100.0
          } else {
            0.0
          }
        } else {
          0.0
        };

        FrontendEPSData {
          id: uuid::Uuid::new_v4().to_string(),
          symbol: data.symbol.clone(),
          name: data.symbol.clone(),
          company_name: data.company_name.clone(),
          current_eps: data.current_eps,
          previous_eps: data.quarterly_data
            .get(1)
            .map(|q| q.actual_eps)
            .unwrap_or(0.0),
          growth_rate: qoq_growth,
          qoq_growth,
          market_cap: data.market_cap_basic as i64,
          price_current: data.price_current,
          volume: data.volume as i64,
          country: data.country.clone(),
          sector: data.sector.clone(),
          ranking_score: 0.0,
          last_updated: chrono::Utc::now(),
        }
      })
      .collect()
  }
}
