// WebSocket Connection Management
// Handles TradingView WebSocket connection setup and messaging

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::SinkExt;
use serde_json::Value;
use tracing::info;
use base64::prelude::*;
use rand;
use url;
use http;

use crate::core::errors::AppError;

/// Establish WebSocket connection to TradingView
pub async fn connect_websocket() -> Result<
  tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
  AppError
> {
  let websocket_url = "wss://data.tradingview.com/socket.io/websocket";

  let url = url::Url::parse(websocket_url)
    .map_err(|e| AppError::network_error(format!("Invalid WebSocket URL: {}", e)))?;

  let request = http::Request::builder()
    .method("GET")
    .uri(url.as_str())
    .header("Host", url.host_str().unwrap_or("data.tradingview.com"))
    .header("Connection", "Upgrade")
    .header("Upgrade", "websocket")
    .header("Sec-WebSocket-Version", "13")
    .header("Sec-WebSocket-Key", BASE64_STANDARD.encode(rand::random::<[u8; 16]>()))
    .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    .header("Origin", "https://www.tradingview.com")
    .header("Cache-Control", "no-cache")
    .body(())
    .map_err(|e| AppError::network_error(format!("Failed to build WebSocket request: {}", e)))?;

  let (ws_stream, _) = connect_async(request).await
    .map_err(|e| AppError::network_error(format!("WebSocket connection failed: {}", e)))?;

  info!("✅ Successfully connected to TradingView WebSocket");
  Ok(ws_stream)
}

/// Send message to WebSocket
pub async fn send_message(
  write: &mut futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message
  >,
  msg: &Value
) -> Result<(), AppError> {
  let message_str = format_tradingview_message(msg);

  write.send(Message::Text(message_str.clone())).await
    .map_err(|e| AppError::network_error(format!("Failed to send message: {}", e)))?;

  Ok(())
}

/// Format TradingView message for sending
fn format_tradingview_message(msg: &Value) -> String {
  format!("~m~{}~m~{}", serde_json::to_string(msg).unwrap().len(), serde_json::to_string(msg).unwrap())
}

/// Parse incoming TradingView messages
pub fn parse_tradingview_message(text: &str) -> Vec<Value> {
  let mut messages = Vec::new();
  let parts: Vec<&str> = text.split("~m~").filter(|s| !s.is_empty()).collect();

  let mut i = 0;
  while i < parts.len() {
    if let Ok(_length) = parts[i].parse::<usize>() {
      if i + 1 < parts.len() {
        if let Ok(json) = serde_json::from_str::<Value>(parts[i + 1]) {
          messages.push(json);
        }
        i += 2;
      } else {
        break;
      }
    } else {
      i += 1;
    }
  }

  messages
}

/// Generate random ID for sessions
pub fn generate_id(length: usize) -> String {
  use uuid::Uuid;
  Uuid::new_v4()
    .to_string()
    .replace("-", "")
    .chars()
    .take(length)
    .collect()
}
