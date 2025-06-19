use tokio::sync::broadcast;
use crate::stock::common::{WebSocketClient, StockServiceError};
use super::models::{PriceDataRequest, PriceDataResponse, WebSocketMessage, Candlestick};

pub struct PriceDataService {
    ws_client: WebSocketClient,
    price_tx: broadcast::Sender<PriceDataResponse>,
    candle_tx: broadcast::Sender<Candlestick>,
}

use crate::config::Config;

impl PriceDataService {
    pub fn new(config: &Config) -> Self {
        let (price_tx, _) = broadcast::channel(100);
        let (candle_tx, _) = broadcast::channel(100);
        
        Self {
            ws_client: WebSocketClient::new(
                "wss://stream.tradingview.com/socket.io/websocket",
                &config.tradingview_auth_token
            ),
            price_tx,
            candle_tx,
        }
    }

    pub async fn subscribe_to_symbols(
        &self,
        symbols: Vec<String>,
        interval: String,
    ) -> Result<(), StockServiceError> {
        let request = PriceDataRequest {
            symbols,
            interval,
        };

        let connection = self.ws_client
            .connect::<WebSocketMessage>()
            .await?;
        
        // Send subscription request
        connection.send_message(&WebSocketMessage {
            message_type: "subscribe".to_string(),
            data: serde_json::to_value(request).map_err(StockServiceError::ParserError)?,
        }).await?;

        // Start receiving messages
        let mut receiver = connection.start_receive_loop().await?;
        
        // Spawn a task to handle incoming messages
        let price_tx = self.price_tx.clone();
        let candle_tx = self.candle_tx.clone();
        
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message.message_type.as_str() {
                    "price" => {
                        if let Ok(price_data) = serde_json::from_value::<PriceDataResponse>(message.data) {
                            if price_tx.send(price_data).is_err() {
                                tracing::error!("Failed to broadcast price data");
                                break;
                            }
                        }
                    }
                    "candle" => {
                        if let Ok(candle) = serde_json::from_value::<Candlestick>(message.data) {
                            if candle_tx.send(candle).is_err() {
                                tracing::error!("Failed to broadcast candlestick data");
                                break;
                            }
                        }
                    }
                    _ => {} // Ignore other message types
                }
            }
        });

        Ok(())
    }

    pub fn subscribe_to_price_updates(&self) -> broadcast::Receiver<PriceDataResponse> {
        self.price_tx.subscribe()
    }

    pub fn subscribe_to_candlesticks(&self) -> broadcast::Receiver<Candlestick> {
        self.candle_tx.subscribe()
    }
}

impl Default for PriceDataService {
    fn default() -> Self {
        panic!("PriceDataService::default is not implemented. Use PriceDataService::new with Config instead.");
    }
}
