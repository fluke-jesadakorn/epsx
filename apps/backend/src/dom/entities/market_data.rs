// Market data domain entities
use serde::{Deserialize, Serialize};
use chrono::{TimeZone, Utc};
use utoipa::ToSchema;

/// Stock screening result with financial metrics
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct StockScreeningResult {
    pub symbol: String,
    pub name: String,
    pub value_index: String,
    pub growth_rate: String,
    pub activity_score: String,
    pub market_size: String,
    pub growth_factor: String,
    pub sector: String,
    pub country: String,
    pub exchange: String,
    pub currency: String,
    pub metric_score: String,
    pub growth_indicator: String,
    pub current_metric: String,
    pub predicted_metric: String,
    pub last_analysis_date: String,
    pub next_analysis_date: String,
    pub entry_phase: PhaseInfo,
    pub phase_status: PhaseStatus,
    pub start_buy: Option<ActionStatus>,
    pub start_action: Option<ActionType>,
    pub eps_growth: Option<f64>,
    pub last_earnings_date: Option<String>,
}

/// Phase information for stock analysis
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PhaseInfo {
    pub date: String,
    pub active: bool,
}

/// Phase status information
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PhaseStatus {
    pub date: String,
    #[serde(rename = "type")]
    pub phase_type: PhaseType,
    pub active: bool,
}

/// Phase type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PhaseType {
    Monitor,
}

/// Action status
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ActionStatus {
    pub active: bool,
}

/// Action type information
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ActionType {
    #[serde(rename = "type")]
    pub action_type: String,
    pub active: bool,
}

/// Number formatting utilities for financial data
pub trait FinancialFormatter {
    fn format_large_number(num: f64) -> String {
        if num >= 1e12 {
            format!("{:.2}T", num / 1e12)
        } else if num >= 1e9 {
            format!("{:.2}B", num / 1e9)
        } else if num >= 1e6 {
            format!("{:.2}M", num / 1e6)
        } else {
            format!("{:.2}", num)
        }
    }

    fn format_number(num: Option<f64>) -> String {
        match num {
            Some(n) if !n.is_nan() => format!("{:.2}", n),
            _ => "N/A".to_string(),
        }
    }

    fn format_date(timestamp: Option<i64>) -> String {
        timestamp
            .map(|ts| {
                Utc.timestamp_opt(ts, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or("N/A".to_string())
            })
            .unwrap_or("N/A".to_string())
    }
}

impl FinancialFormatter for StockScreeningResult {}

impl StockScreeningResult {
    /// Get analysis phases from timestamp data
    pub fn get_analysis_phases(last: i64, next: i64) -> (PhaseInfo, PhaseStatus) {
        let last_date = Self::format_date(Some(last));
        let next_date = Self::format_date(Some(next));
        
        let entry_phase = PhaseInfo {
            date: last_date.clone(),
            active: last > 0,
        };

        let phase_status = PhaseStatus {
            date: next_date,
            phase_type: PhaseType::Monitor,
            active: next > 0,
        };

        (entry_phase, phase_status)
    }
}

/// TradingView API response structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingViewResponse {
    #[serde(rename = "totalCount")]
    pub total_count: i32,
    pub data: Vec<TradingViewStock>,
}

/// TradingView stock data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradingViewStock {
    pub s: String, // symbol
    pub d: Vec<StockDataField>,
}

/// Stock data field (can be string, number, boolean, array, object, or null)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum StockDataField {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<serde_json::Value>),
    Object(serde_json::Map<String, serde_json::Value>),
    Null,
}

/// Number formatter utility
pub struct NumberFormatter;

impl NumberFormatter {
    pub fn format_number(field: &StockDataField) -> String {
        match field {
            StockDataField::Number(n) => {
                if n.is_nan() { "N/A".to_string() } else { format!("{:.2}", n) }
            },
            StockDataField::Integer(i) => i.to_string(),
            StockDataField::String(s) => s.clone(),
            StockDataField::Boolean(b) => b.to_string(),
            StockDataField::Array(_) => "Array".to_string(),
            StockDataField::Object(_) => "Object".to_string(),
            StockDataField::Null => "N/A".to_string(),
        }
    }
}

/// Table data metrics for market analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableDataMetrics {
    pub columns: Vec<String>,
    pub data: Vec<Vec<StockDataField>>,
}

/// Quote session creation request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteSessionCreate {
    pub session_id: String,
    pub symbols: Vec<String>,
}

/// Market data service errors (domain-level)
#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("External API error: {0}")]
    ExternalApiError(String),
    
    #[error("Data parsing error: {0}")]
    ParsingError(String),
    
    #[error("Network connectivity error: {0}")]
    NetworkError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("IO error: {0}")]
    IOError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<crate::infra::services::websocket::WebSocketError> for MarketDataError {
    fn from(err: crate::infra::services::websocket::WebSocketError) -> Self {
        match err {
            crate::infra::services::websocket::WebSocketError::ConnectionError(msg) => MarketDataError::NetworkError(msg),
            crate::infra::services::websocket::WebSocketError::AuthenticationError(msg) => MarketDataError::ExternalApiError(msg),
            crate::infra::services::websocket::WebSocketError::ParsingError(msg) => MarketDataError::ParsingError(msg),
            crate::infra::services::websocket::WebSocketError::NetworkError(msg) => MarketDataError::NetworkError(msg),
        }
    }
}