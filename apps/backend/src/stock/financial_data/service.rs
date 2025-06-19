use tokio::sync::broadcast;
use crate::{config::Config, stock::common::{WebSocketClient, StockServiceError}};
use super::models::{FinancialDataRequest, FinancialDataResponse, WebSocketMessage};
use tracing::info;

pub struct FinancialDataService {
    ws_client: WebSocketClient,
    data_tx: broadcast::Sender<FinancialDataResponse>,
}

impl FinancialDataService {
    pub fn new(config: &Config) -> Self {
        let (data_tx, _) = broadcast::channel(100);

        info!("Initializing FinancialDataService with TradingView auth token");

        Self {
            ws_client: WebSocketClient::new(
                "wss://data.tradingview.com/socket.io/websocket?type=chart",
                &config.tradingview_auth_token
            ),
            data_tx,
        }
    }

    pub async fn subscribe_to_symbols(&self, symbols: Vec<String>) -> Result<(), StockServiceError> {
        let fields = vec![
            "pe_ratio".to_string(),
            "market_cap".to_string(),
            "volume".to_string(),
            "eps".to_string(),
            "dividend_yield".to_string(),
            "revenue".to_string(),
            "net_income".to_string(),
            "debt_to_equity".to_string(),
        ];

        let request = FinancialDataRequest {
            symbols,
            fields,
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
        let data_tx = self.data_tx.clone();
        
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                if message.message_type == "data" {
                    if let Ok(data) = serde_json::from_value::<FinancialDataResponse>(message.data) {
                        if data_tx.send(data).is_err() {
                            tracing::error!("Failed to broadcast financial data");
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn subscribe_to_updates(&self) -> broadcast::Receiver<FinancialDataResponse> {
        self.data_tx.subscribe()
    }
}

impl Default for FinancialDataService {
    fn default() -> Self {
        panic!("FinancialDataService::default is not implemented. Use FinancialDataService::new with Config instead.");
    }
}
