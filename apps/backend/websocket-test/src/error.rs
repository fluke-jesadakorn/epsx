use tokio_tungstenite::tungstenite::http;

#[derive(Debug, thiserror::Error)]
pub enum StockServiceError {
    #[error("Parser error: {0}")] ParserError(#[from] serde_json::Error),
    #[error("WebSocket error: {0}")] WebSocketError(String),
    #[error("HTTP error: {0}")] HttpError(#[from] http::Error),
    #[error("Network error: {0}")] NetworkError(#[from] reqwest::Error),
}
