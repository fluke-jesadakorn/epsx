use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, 
    WebSocketStream, tungstenite::handshake::client::{Request, generate_key}
};
use tokio::sync::{mpsc, Mutex};
use url::Url;
use futures_util::stream::{SplitSink, SplitStream};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT, ORIGIN};
use tokio::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::error::StockServiceError;
use crate::models::{WebSocketMessage, QuoteSymbolMessage, QuoteSymbolPayload};
use tracing::{error, info, debug};
use serde_json::json;

pub struct WsClient {
    url: String,
    auth_token: String,
}

impl WsClient {
    pub fn new(url: &str, auth_token: &str) -> Self {
        Self { url: url.to_string(), auth_token: auth_token.to_string() }
    }

    async fn get_session_cookies(&self) -> Result<String, StockServiceError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
        ));
        headers.insert(ORIGIN, HeaderValue::from_static("https://www.tradingview.com"));
        headers.insert(COOKIE, HeaderValue::from_str(&format!("sessionid={}", self.auth_token))
            .map_err(|e| StockServiceError::WebSocketError(e.to_string()))?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| StockServiceError::NetworkError(e))?;

        let response = client.get("https://www.tradingview.com/chart/")
            .send().await
            .map_err(|e| StockServiceError::NetworkError(e))?;

        Ok(response.headers()
            .get_all("set-cookie")
            .into_iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("; "))
    }

    fn format_tv_message(msg: &str) -> String {
        format!("~m~{}~m~{}", msg.len(), msg)
    }

    fn create_tv_message(method: &str, params: Vec<String>) -> String {
        Self::format_tv_message(&json!({"m": method, "p": params}).to_string())
    }

    pub async fn connect<T>(&self) -> Result<WsConn<T>, StockServiceError>
        where T: DeserializeOwned + Send + std::fmt::Debug + Clone + 'static
    {
        // First, get session cookies
        let cookies = self.get_session_cookies().await?;

        let url = Url::parse(&self.url).map_err(|e|
            StockServiceError::WebSocketError(e.to_string())
        )?;

        // Create custom request with headers
        let request = Request::builder()
            .uri(url.as_str())
            .header("Host", url.host_str().unwrap_or("data.tradingview.com"))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Origin", "https://www.tradingview.com")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36"
            )
            .header("Cookie", cookies)
            .body(())?;

        info!("Establishing WebSocket connection...");
        let (ws_stream, _response) = connect_async(request).await.map_err(|e| {
            if e.to_string().contains("403 Forbidden") {
                StockServiceError::WebSocketError(
                    "TradingView authentication failed. Please ensure auth token is valid.".to_string()
                )
            } else {
                StockServiceError::WebSocketError(e.to_string())
            }
        })?;

        let (tx, _rx) = mpsc::channel(100);
        let connection = WsConn::new(ws_stream, tx);

        // Initial setup for unauthorized user
        let unauthorized_token = json!({"m":"set_auth_token","p":["unauthorized_user_token"]});
        connection.send_raw_message(
            &Self::format_tv_message(&unauthorized_token.to_string())
        ).await?;

        // Small delay to ensure auth message is processed
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Start ping task in the background
        let connection_clone = connection.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(15)).await;
                let ping_msg = Self::format_tv_message(&json!({"m":"ping"}).to_string());
                if connection_clone.send_raw_message(&ping_msg).await.is_err() {
                    break;
                }
            }
        });
        Ok(connection)
    }

    pub fn gen_sess_id() -> String {
        format!("cs_{}_{}", chrono::Utc::now().timestamp_millis(), uuid::Uuid::new_v4().simple())
    }

    pub fn msg_chart_sess(sess: &str) -> String {
        Self::create_tv_message("chart_create_session", vec![sess.to_string(), "".to_string()])
    }

    pub fn msg_resolve_sym(sess: &str, sym: &str) -> String {
        Self::create_tv_message("resolve_symbol", vec![
            sess.to_string(),
            "sds_sym_1".to_string(),
            format!("={{\"adjustment\":\"splits\",\"symbol\":\"{}\"}}", sym)
        ])
    }

    pub fn msg_series(sess: &str) -> String {
        Self::create_tv_message("create_series", vec![
            sess.to_string(), "sds_1".to_string(), "s1".to_string(),
            "sds_sym_1".to_string(), "D".to_string(), "5".to_string(), "".to_string()
        ])
    }

    pub fn gen_quote_sess() -> String {
        format!("qs_{}", uuid::Uuid::new_v4().simple())
    }

    pub fn msg_quote_sess(sess: &str) -> String {
        Self::create_tv_message("quote_create_session", vec![sess.to_string()])
    }

    pub fn msg_quote_add_adj(sess: &str, sym: &str) -> String {
        Self::create_tv_message("quote_add_symbols", vec![
            sess.to_string(),
            format!("={{\"adjustment\":\"splits\",\"symbol\":\"{}\"}}", sym)
        ])
    }

    pub fn msg_quote_fast(sess: &str, sym: &str) -> String {
        Self::create_tv_message("quote_fast_symbols", vec![
            sess.to_string(),
            format!("={{\"adjustment\":\"splits\",\"symbol\":\"{}\"}}", sym)
        ])
    }

    pub fn msg_quote_add(sess: &str, sym: &str) -> String {
        Self::create_tv_message("quote_add_symbols", vec![sess.to_string(), sym.to_string()])
    }

    pub fn msg_quote_rm(sess: &str, sym: &str) -> String {
        Self::create_tv_message("quote_remove_symbols", vec![sess.to_string(), sym.to_string()])
    }
}

pub struct WsConn<T> {
    write: Arc<
        Mutex<
            SplitSink<
                WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
                Message
            >
        >
    >,
    read: Arc<Mutex<SplitStream<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>>,
    message_tx: mpsc::Sender<T>,
    is_connected: Arc<AtomicBool>,
}

impl<T> Clone for WsConn<T> {
    fn clone(&self) -> Self {
        Self {
            write: Arc::clone(&self.write),
            read: Arc::clone(&self.read),
            message_tx: self.message_tx.clone(),
            is_connected: Arc::clone(&self.is_connected),
        }
    }
}

impl<T> WsConn<T> where T: DeserializeOwned + Send + std::fmt::Debug + Clone + 'static {
    async fn switch_symbol(
        &self,
        session_id: &str,
        current_symbol: &str,
        target_symbol: &str
    ) -> Result<(), StockServiceError> {
        // Remove current symbol
        let remove_msg = WsClient::msg_quote_rm(
            session_id,
            current_symbol
        );
        self.send_raw_message(&remove_msg).await?;

        // Add target symbol
        let add_msg = WsClient::msg_quote_add(
            session_id,
            target_symbol
        );
        self.send_raw_message(&add_msg).await?;

        Ok(())
    }

    async fn handle_qsd_message(
        &self,
        json: &serde_json::Value,
        pending_switches: &Arc<Mutex<std::collections::HashMap<String, (String, chrono::DateTime<chrono::Utc>)>>>
    ) -> Result<(), StockServiceError> {
        if let Ok(quote_msg) = serde_json::from_value::<QuoteSymbolMessage>(json.clone()) {
            if quote_msg.m == "qsd" {
                let session_id = quote_msg.p.get(0).and_then(|v| v.as_str());
                let symbol_data = quote_msg.p.get(1)
                    .and_then(|p| serde_json::from_value::<QuoteSymbolPayload>(p.clone()).ok());

                if let (Some(session_id), Some(symbol_data)) = (session_id, symbol_data) {
                    if let Some(original_name) = symbol_data.v.as_object()
                        .and_then(|v| v.get("original_name"))
                        .and_then(|v| v.as_str())
                    {
                        if symbol_data.n != original_name {

                            // Check if this was a pending switch we were waiting for
                            let mut switches = pending_switches.lock().await;
                            if switches.contains_key(&symbol_data.n) {
                                switches.remove(&symbol_data.n);
                                
                                // Perform the switch using data from websocket
                                return self.switch_symbol(
                                    session_id,
                                    &symbol_data.n,
                                    original_name
                                ).await;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn new(
        ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        message_tx: mpsc::Sender<T>
    ) -> Self {
        let (write, read) = ws_stream.split();
        Self {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
            message_tx,
            is_connected: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn send_raw_message(&self, message: &str) -> Result<(), StockServiceError> {
        if !self.is_connected.load(Ordering::SeqCst) {
            return Err(StockServiceError::WebSocketError("WebSocket is disconnected".to_string()));
        }

        if !message.starts_with("~m~") {
            return Err(StockServiceError::WebSocketError("Invalid message format".to_string()));
        }

        tokio::time::sleep(Duration::from_millis(500)).await;

        match self.write.lock().await.send(Message::Text(message.to_string())).await {
            Ok(_) => Ok(()),
            Err(e) => {
                self.is_connected.store(false, Ordering::SeqCst);
                Err(StockServiceError::WebSocketError(e.to_string()))
            }
        }
    }

    pub fn start_receive_loop(self) -> Result<mpsc::Receiver<T>, StockServiceError> {
        let (tx, rx) = mpsc::channel(100);
        let pending_switches = Arc::new(Mutex::new(std::collections::HashMap::new()));

        tokio::spawn(async move {
            while let Some(Ok(message)) = self.read.lock().await.next().await {
                match message {
                    Message::Text(text) if text.starts_with("~m~") => {
                        // Log raw message for debugging
                        debug!("Received raw WebSocket text message: {}", text);

                        // Handle heartbeat
                        if text.contains("~h~") {
                            if let Err(e) = self.write.lock().await.send(Message::Text(text.replace("~h~", "~m~"))).await {
                                error!("Heartbeat failed: {}", e);
                                self.is_connected.store(false, Ordering::SeqCst);
                                break;
                            }
                            continue;
                        }

                        // Parse and handle message
                        if let Some(json) = WebSocketMessage::parse_tradingview(&text) {
                            // Log full payload for qsd and timescale_update messages
                            if let Some(msg_type) = json.get("m").and_then(|m| m.as_str()) {
                                match msg_type {
                                    "qsd" => debug!("QSD message payload: {:?}", json),
                                    "timescale_update" => debug!("Timescale update payload: {:?}", json),
                                    _ => {}
                                }
                            }
                            // Process received message without verbose logging

                            // Handle qsd messages
                            if let Ok(quote_msg) = serde_json::from_value::<QuoteSymbolMessage>(json.clone()) {
                                if quote_msg.m == "qsd" && self.handle_qsd_message(&json, &pending_switches).await.is_err() {
                                    error!("Failed to handle qsd message");
                                }
                            }

                            // Forward message and track symbol requests
                            if let Ok(parsed) = serde_json::from_value::<T>(json.clone()) {
                                debug!("Parsed message type: {:?}", parsed);
                                if tx.send(parsed).await.is_err() { 
                                    error!("Failed to send parsed message through channel");
                                    break; 
                                }

                                if let Ok(msg) = serde_json::from_value::<QuoteSymbolMessage>(json) {
                                    if msg.m == "quote_add_symbols" && msg.p.len() > 1 {
                                        // Extract values from msg.p first
                                        let sid = msg.p.get(0).and_then(|v| v.as_str());
                                        let sym = msg.p.get(1).and_then(|v| v.as_str());
                                        
                                        if let (Some(sid), Some(sym)) = (sid, sym) {
                                            let sid = sid.to_string();
                                            let sym = sym.to_string();
                                            let switches = Arc::clone(&pending_switches);
                                            pending_switches.lock().await.insert(sym.clone(), 
                                                (sid, chrono::Utc::now()));

                                            let sym = sym.clone();  // Clone for the async move block
                                            tokio::spawn(async move {
                                                tokio::time::sleep(Duration::from_secs(5)).await;
                                                let now = chrono::Utc::now().timestamp();
                                                if let Some((_, created_at)) = switches.lock().await.get(&sym) {
                                                    if now - created_at.timestamp() >= 5 {
                                                        switches.lock().await.remove(&sym);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Message::Close(frame) => {
                        if let Some(f) = frame {
                            info!("WebSocket closed: {:?}", f.code);
                        }
                        self.is_connected.store(false, Ordering::SeqCst);
                        break;
                    }
                    Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
                    _ => (),
                }
            }
        });

        Ok(rx)
    }
}
