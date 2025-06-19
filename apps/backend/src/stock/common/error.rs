use axum::{response::Response, http::StatusCode};
use axum::response::IntoResponse;
use mongodb::error::Error as MongoError;

#[derive(Debug, thiserror::Error, utoipa::ToSchema)]
#[schema(example = json!({"error": "TradingView API error: Connection failed"}))]
pub enum StockServiceError {
    #[error("TradingView API error: {0}")]
    TradingViewError(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Parser error: {0}")]
    ParserError(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    DatabaseError(MongoError),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

impl IntoResponse for StockServiceError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::TradingViewError(_) => StatusCode::BAD_GATEWAY,
            Self::NetworkError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ParserError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebSocketError(_) => StatusCode::BAD_GATEWAY,
        };
        
        tracing::error!("Stock service error: {}", self);
        (status, self.to_string()).into_response()
    }
}

impl From<MongoError> for StockServiceError {
    fn from(error: MongoError) -> Self {
        Self::DatabaseError(error)
    }
}
