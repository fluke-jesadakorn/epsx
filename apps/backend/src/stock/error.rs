use axum::{response::Response, http::StatusCode};
use axum::response::IntoResponse;

#[derive(Debug, thiserror::Error)]
pub enum StockServiceError {
    #[error("TradingView API error: {0}")]
    TradingViewError(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Parser error: {0}")]
    ParserError(#[from] serde_json::Error),
}

impl IntoResponse for StockServiceError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::TradingViewError(_) => StatusCode::BAD_GATEWAY,
            Self::NetworkError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ParserError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        tracing::error!("Stock service error: {}", self);
        (status, self.to_string()).into_response()
    }
}
