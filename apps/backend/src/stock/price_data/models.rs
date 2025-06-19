use serde::{Deserialize, Serialize};
use crate::stock::common::NumberFormatter;

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceDataRequest {
    pub symbols: Vec<String>,
    pub interval: String, // e.g., "1m", "5m", "1h", "1d"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceDataResponse {
    pub symbol: String,
    pub price_data: PriceData,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceData {
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub volume: f64,
}

impl NumberFormatter for PriceData {}

impl PriceData {
    #[allow(dead_code)]
    pub fn format(&self) -> FormattedPriceData {
        FormattedPriceData {
            price: Self::format_number(Some(self.price)),
            change: format!("{:+.2}", self.change),
            change_percent: format!("{:+.2}%", self.change_percent),
            high: Self::format_number(Some(self.high)),
            low: Self::format_number(Some(self.low)),
            open: Self::format_number(Some(self.open)),
            volume: Self::format_large_number(self.volume),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FormattedPriceData {
    pub price: String,
    pub change: String,
    pub change_percent: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub volume: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candlestick {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
}
