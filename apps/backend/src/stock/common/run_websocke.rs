use super::{ WebSocketClient, StockServiceError };
use crate::stock::financial_data::models::WebSocketMessage;
use std::env;
use dotenv::dotenv;
use tracing::info;

pub async fn run_test_websocket() -> Result<(), StockServiceError> {
    dotenv().ok();
    info!("Starting WebSocket test client...");

    // Get auth token from environment
    let auth_token = String::from(
        "eyJhbGciOiJSUzUxMiIsImtpZCI6IkdaeFUiLCJ0eXAiOiJKV1QifQ.eyJ1c2VyX2lkIjoxOTQ0ODE3MiwiZXhwIjoxNzQ0NzI3NTc5LCJpYXQiOjE3NDQ3MTMxNzksInBsYW4iOiIiLCJkZWNsYXJlZF9zdGF0dXMiOiJub25fcHJvIiwiZXh0X2hvdXJzIjoxLCJwZXJtIjoiIiwic3R1ZHlfcGVybSI6IiIsIm1heF9zdHVkaWVzIjoyLCJtYXhfZnVuZGFtZW50YWxzIjoxLCJtYXhfY2hhcnRzIjoxLCJtYXhfYWN0aXZlX2FsZXJ0cyI6MSwibWF4X3N0dWR5X29uX3N0dWR5IjoxLCJmaWVsZHNfcGVybWlzc2lvbnMiOltdLCJtYXhfb3ZlcmFsbF9hbGVydHMiOjIwMDAsIm1heF9hY3RpdmVfcHJpbWl0aXZlX2FsZXJ0cyI6NSwibWF4X2FjdGl2ZV9jb21wbGV4X2FsZXJ0cyI6MSwibWF4X2Nvbm5lY3Rpb25zIjoyfQ.Ghkec92So6sE2dwhabfbYeyz0nAtdq6Xr8AofdnSZiONqljFhx4QHaiTY05nibtye1dT70fQtDOGdgKTnGC4XCVzFmXRKPgc5Rf6yKvtecW1AgD-OaYzC5wWGu8R3c-DPnuVQ6PENATwb2tNb48vbiqsSLm9_ZxjIu5FRv8cfuE"
    );

    // Create WebSocket client
    let client = WebSocketClient::new(
        "wss://data.tradingview.com/socket.io/websocket?type=chart",
        &auth_token
    );

    info!("Connecting to WebSocket server...");

    // Connect to WebSocket server
    let connection = client.connect::<WebSocketMessage>().await?;

    // Send test subscription message
    let test_message = WebSocketMessage {
        message_type: "subscribe".to_string(),
        data: serde_json::json!({
            "symbols": ["AAPL", "GOOGL"],
            "fields": ["pe_ratio", "market_cap", "volume"]
        }),
    };

    info!("Sending test subscription message...");
    connection.send_message(&test_message).await?;

    // Start receiving messages
    info!("Starting message receive loop...");
    let mut receiver = connection.start_receive_loop().await?;

    // Process received messages for 60 seconds
    let timeout = tokio::time::Duration::from_secs(60);
    let start = tokio::time::Instant::now();

    while let Some(message) = receiver.recv().await {
        info!("Received message: {:?}", message);

        if start.elapsed() >= timeout {
            info!("Debug session timeout reached");
            break;
        }
    }

    info!("WebSocket test client finished");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_connection() {
        // Only run this test when explicitly enabled via environment variable
        if env::var("RUN_WEBSOCKET_TEST").is_err() {
            println!("Skipping websocket test. Set RUN_WEBSOCKET_TEST=1 to enable.");
            return;
        }

        let result = run_test_websocket().await;
        assert!(result.is_ok(), "WebSocket test failed: {:?}", result);
    }
}