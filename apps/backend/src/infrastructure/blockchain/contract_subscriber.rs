// WebSocket-based contract event subscriber
// Real-time payment event monitoring with reconnection and HTTP fallback

use crate::config::contracts::{Chain, ChainContractConfig, PAYMENT_EVENT_TOPIC};
use crate::domain::shared_kernel::app_error::AppError;
use crate::infrastructure::blockchain::{PaymentEvent, parse_payment_event, PaymentVerifier};

use ethers::types::U256;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

/// WebSocket subscription state
#[derive(Debug, Clone, PartialEq)]
pub enum SubscriptionState {
    Disconnected,
    Connecting,
    Connected,
    Subscribed,
    Failed,
}

/// JSON-RPC request/response
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(default)]
    result: serde_json::Value,
    #[serde(default)]
    error: Option<JsonRpcError>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<JsonRpcParams>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcParams {
    #[serde(default)]
    subscription: String,
    #[serde(default)]
    result: serde_json::Value,
}

/// Subscription filter for logs
#[derive(Debug, Serialize)]
struct LogFilter {
    address: String,
    topics: Vec<Option<String>>,
}

/// Contract event subscriber with WebSocket
pub struct ContractSubscriber {
    chain: Chain,
    config: ChainContractConfig,
    payment_verifier: Arc<PaymentVerifier>,
    state: SubscriptionState,
    _subscription_id: Option<String>,
    _reconnect_attempts: u32,
    _max_reconnect_attempts: u32,
    _backoff_seconds: u64,
}

impl ContractSubscriber {
    /// Create new contract subscriber
    pub fn new(
        config: ChainContractConfig,
        supported_tokens: Vec<String>,
    ) -> Result<Self, AppError> {
        let payment_verifier = Arc::new(PaymentVerifier::new(
            config.http_url.clone(),
            config.contract_address
                .as_ref()
                .map(|a| format!("0x{}", hex::encode(a.as_bytes())))
                .unwrap_or_default(),
            supported_tokens,
        )?);

        Ok(Self {
            chain: config.chain,
            config,
            payment_verifier,
            state: SubscriptionState::Disconnected,
            _subscription_id: None,
            _reconnect_attempts: 0,
            _max_reconnect_attempts: 10,
            _backoff_seconds: 1,
        })
    }

    /// Start subscription with callback
    pub async fn subscribe<F>(&mut self, callback: F) -> Result<(), AppError>
    where
        F: Fn(PaymentEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        info!("🔗 Starting {} contract subscriber", self.chain);

        let callback = Arc::new(callback);

        // Try WebSocket first
        let mut ws_result = self.run_websocket(Arc::clone(&callback)).await;

        // If WebSocket fails, try backup
        if ws_result.is_err() {
            if let Some(backup_url) = &self.config.ws_backup_url {
                warn!("⚠️ Primary WebSocket failed, trying backup: {}", backup_url);
                let backup_config = ChainContractConfig {
                    ws_url: backup_url.clone(),
                    ws_backup_url: None,
                    ..self.config.clone()
                };
                match Self::new(backup_config, vec![]) {
                    Ok(mut backup_sub) => {
                        ws_result = backup_sub.run_websocket(Arc::clone(&callback)).await;
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to create backup subscriber: {}", e);
                    }
                }
            }
        }

        // If both fail, fall back to HTTP polling
        if ws_result.is_err() {
            warn!("⚠️ WebSocket unavailable, falling back to HTTP polling for {}", self.chain);
            self.run_http_polling(callback).await
        } else {
            ws_result
        }
    }

    /// Run WebSocket subscription
    async fn run_websocket<F>(&mut self, callback: Arc<F>) -> Result<(), AppError>
    where
        F: Fn(PaymentEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.state = SubscriptionState::Connecting;

        // Parse contract address
        let contract_addr = self.config.contract_address.ok_or_else(|| {
            AppError::infrastructure_error("No contract address configured")
        })?;

        // Connect with timeout
        let ws_url = self.config.ws_url.clone();
        let url = url::Url::parse(&ws_url)
            .map_err(|e| AppError::infrastructure_error(format!("Invalid WebSocket URL: {}", e)))?;

        info!("📡 Connecting to WebSocket: {}", ws_url);
        let (ws_stream, _) = tokio::time::timeout(
            Duration::from_secs(10),
            connect_async(url),
        )
        .await
        .map_err(|_| AppError::infrastructure_error("WebSocket connection timeout".to_string()))?
        .map_err(|e| AppError::infrastructure_error(format!("WebSocket connection failed: {}", e)))?;

        info!("✅ WebSocket connected for {}", self.chain);
        self.state = SubscriptionState::Connected;

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Subscribe to logs
        let contract_hex = format!("0x{}", hex::encode(contract_addr.as_bytes()));
        let filter = LogFilter {
            address: contract_hex.clone(),
            topics: vec![Some(PAYMENT_EVENT_TOPIC.to_string()), None, None, None],
        };

        let subscribe_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_subscribe".to_string(),
            params: json!(["logs", filter]),
            id: 1,
        };

        let subscribe_msg = serde_json::to_string(&subscribe_request)
            .map_err(|e| AppError::infrastructure_error(format!("JSON serialization error: {}", e)))?;

        ws_sender.send(WsMessage::Text(subscribe_msg)).await
            .map_err(|e| AppError::infrastructure_error(format!("Failed to send subscription: {}", e)))?;

        info!("🔔 Subscription request sent for contract {} on {}", contract_hex, self.chain);

        // Process messages
        let verifier = Arc::clone(&self.payment_verifier);
        let chain = self.chain;
        let mut current_state = SubscriptionState::Connected;

        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(WsMessage::Text(text)) => {
                    if let Err(e) = Self::handle_message(
                        &text,
                        &chain,
                        &verifier,
                        Arc::clone(&callback),
                        &mut current_state,
                    ).await {
                        error!("❌ Error handling message: {}", e);
                    }
                }
                Ok(WsMessage::Ping(data)) => {
                    if let Err(e) = ws_sender.send(WsMessage::Pong(data)).await {
                        error!("❌ Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    warn!("🔌 WebSocket closed by server");
                    break;
                }
                Err(e) => {
                    error!("❌ WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Err(AppError::infrastructure_error("WebSocket connection ended".to_string()))
    }

    /// Handle incoming WebSocket message
    async fn handle_message<F>(
        text: &str,
        chain: &Chain,
        verifier: &Arc<PaymentVerifier>,
        callback: Arc<F>,
        state: &mut SubscriptionState,
    ) -> Result<(), AppError>
    where
        F: Fn(PaymentEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>>
            + Send
            + Sync,
    {
        let response: JsonRpcResponse = serde_json::from_str(text)
            .map_err(|e| {
                debug!("Failed to parse JSON (non-subscription message?): {}", e);
                AppError::infrastructure_error(format!("JSON parse error: {}", e))
            })?;

        // Handle subscription confirmation
        if let Some(method) = &response.method {
            if method == "eth_subscription" {
                if let Some(params) = &response.params {
                    if state != &SubscriptionState::Subscribed {
                        *state = SubscriptionState::Subscribed;
                        info!("✅ Subscription active for {}", chain);
                    }

                    // Parse log data
                    if let Ok(log) = Self::parse_log(&params.result) {
                        if let Ok(event) = parse_payment_event(&log) {
                            if event.is_valid() {
                                info!("🎉 Payment event received on {}: tx={}", chain, event.transaction_hash);

                                // Verify and process
                                match verifier.verify_payment(&event).await {
                                    Ok(verification) => {
                                        if verification.is_verified() {
                                            info!("✅ Payment verified: {}", event.unique_id());
                                            if let Err(e) = callback(event).await {
                                                error!("❌ Callback failed: {}", e);
                                            }
                                        } else {
                                            error!("❌ Payment verification failed: {:?}", verification.errors);
                                        }
                                    }
                                    Err(e) => {
                                        error!("❌ Verification error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse log from JSON value
    fn parse_log(value: &serde_json::Value) -> Result<ethers::types::Log, AppError> {
        #[derive(Deserialize)]
        struct LogData {
            #[serde(default)]
            address: String,
            #[serde(default)]
            topics: Vec<String>,
            #[serde(default)]
            data: String,
            #[serde(default)]
            block_number: String,
            #[serde(default)]
            transaction_hash: String,
            #[serde(default)]
            log_index: String,
        }

        let log_data: LogData = serde_json::from_value(value.clone())
            .map_err(|e| AppError::infrastructure_error(format!("Failed to parse log: {}", e)))?;

        let address = log_data.address.parse::<ethers::types::H160>()
            .map_err(|e| AppError::infrastructure_error(format!("Invalid address: {}", e)))?;

        let topics: Vec<ethers::types::H256> = log_data.topics.iter()
            .filter_map(|t| t.parse::<ethers::types::H256>().ok())
            .collect();

        let data = log_data.data.trim_start_matches("0x");
        let data_bytes = hex::decode(data)
            .unwrap_or_default();

        let block_number = log_data.block_number.trim_start_matches("0x");
        let block_num = U256::from_str_radix(block_number, 16)
            .map_err(|e| AppError::infrastructure_error(format!("Invalid block number: {}", e)))?;

        let tx_hash = log_data.transaction_hash.parse::<ethers::types::H256>()
            .map_err(|e| AppError::infrastructure_error(format!("Invalid tx hash: {}", e)))?;

        let log_index = log_data.log_index.trim_start_matches("0x");
        let log_idx = U256::from_str_radix(log_index, 16)
            .map_err(|e| AppError::infrastructure_error(format!("Invalid log index: {}", e)))?;

        Ok(ethers::types::Log {
            address,
            topics,
            data: data_bytes.into(),
            block_number: Some(ethers::types::U64::from(block_num.as_u64())),
            transaction_hash: Some(tx_hash),
            log_index: Some(ethers::types::U256::from(log_idx.as_u64()).into()),
            ..Default::default()
        })
    }

    /// Run HTTP polling fallback
    async fn run_http_polling<F>(&mut self, _callback: Arc<F>) -> Result<(), AppError>
    where
        F: Fn(PaymentEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>>
            + Send
            + Sync,
    {
        warn!("⚠️ HTTP polling not implemented in contract_subscriber - use BscEventListener for polling");
        // The existing BscEventListener already handles HTTP polling
        // This is a fallback placeholder
        Err(AppError::infrastructure_error("Use BscEventListener for HTTP polling".to_string()))
    }

    /// Get current state
    pub fn state(&self) -> &SubscriptionState {
        &self.state
    }

    /// Get chain
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, SubscriptionState::Connected | SubscriptionState::Subscribed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_filter_serialization() {
        let filter = LogFilter {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            topics: vec![Some("0xabc123".to_string()), None, None],
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("0x1234567890"));
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "eth_subscribe".to_string(),
            params: json!(["logs"]),
            id: 1,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("eth_subscribe"));
    }
}
