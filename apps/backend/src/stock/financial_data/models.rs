use serde::{Deserialize, Serialize};
use crate::stock::common::NumberFormatter;

#[derive(Debug, Serialize, Deserialize)]
pub struct FinancialDataRequest {
    pub symbols: Vec<String>,
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinancialDataResponse {
    pub symbol: String,
    pub metrics: FinancialMetrics,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinancialMetrics {
    pub pe_ratio: Option<f64>,
    pub market_cap: Option<f64>,
    pub volume: Option<f64>,
    pub eps: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub revenue: Option<f64>,
    pub net_income: Option<f64>,
    pub debt_to_equity: Option<f64>,
}

impl NumberFormatter for FinancialMetrics {}

impl FinancialMetrics {
    #[allow(dead_code)]
    pub fn format(&self) -> FormattedFinancialMetrics {
        FormattedFinancialMetrics {
            pe_ratio: Self::format_number(self.pe_ratio),
            market_cap: Self::format_large_number(self.market_cap.unwrap_or(0.0)),
            volume: Self::format_large_number(self.volume.unwrap_or(0.0)),
            eps: Self::format_number(self.eps),
            dividend_yield: self.dividend_yield.map_or("N/A".to_string(), |v| format!("{:.2}%", v)),
            revenue: Self::format_large_number(self.revenue.unwrap_or(0.0)),
            net_income: Self::format_large_number(self.net_income.unwrap_or(0.0)),
            debt_to_equity: Self::format_number(self.debt_to_equity),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FormattedFinancialMetrics {
    pub pe_ratio: String,
    pub market_cap: String,
    pub volume: String,
    pub eps: String,
    pub dividend_yield: String,
    pub revenue: String,
    pub net_income: String,
    pub debt_to_equity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
}
