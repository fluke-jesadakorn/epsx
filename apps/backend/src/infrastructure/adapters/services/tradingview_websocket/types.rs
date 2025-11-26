// TradingView WebSocket Data Types
// Core data structures for WebSocket communication and EPS data

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Frontend EPS data format for client consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendEPSData {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub company_name: String,
    pub current_eps: f64,
    pub previous_eps: f64,
    pub growth_rate: f64,
    pub qoq_growth: f64,
    pub market_cap: i64,
    pub price_current: f64,
    pub volume: i64,
    pub country: String,
    pub sector: String,
    pub ranking_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// TradingView WebSocket message structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewMessage {
  pub m: String, // method
  pub p: Vec<Value>, // parameters
  #[serde(skip_serializing_if = "Option::is_none")]
  pub t: Option<i64>, // timestamp
  #[serde(skip_serializing_if = "Option::is_none")]
  pub t_ms: Option<i64>, // timestamp milliseconds
}

/// Quarterly EPS data point (matching Node.js structure exactly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyEPSData {
  pub quarter_number: usize,
  pub period: String, // "2024-Q3", "2025-Q1", etc.
  pub actual_eps: f64,
  pub timestamp: i64, // earnings timestamp (seconds)
  pub estimated_eps: Option<f64>,
  pub is_reported: bool,
  pub beat_estimate: Option<bool>,
  #[serde(rename = "type")]
  pub eps_type: String, // "st4_earnings_study"
  pub source: String,
  pub quarter_end_date: Option<String>,
  pub estimated_earnings_date: Option<i64>, // earnings announcement timestamp
  pub price_data: Option<PriceImpactData>,
  // Legacy fields for backward compatibility
  pub eps: f64,
  pub quarter_name: String,
}

/// Price impact data around earnings announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpactData {
  pub pre_earnings_price: f64,
  pub post_earnings_price: f64,
  pub price_change: f64,
  pub percent_change: f64,
  pub earnings_impact: String, // "positive" or "negative"
  pub days_before: i32,
  pub days_after: i32,
  pub volume_before: i64,
  pub volume_after: i64,
  pub volume_change: String, // "increased" or "decreased"
  pub data_quality: String, // "excellent", "good", "fair"
}

/// Price data point extracted from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
  pub timestamp: i64,
  pub date: String,
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
  pub volume: i64,
}

/// EPS data extracted from WebSocket with price correlation
#[derive(Debug, Clone, Serialize)]
pub struct EPSWebSocketData {
  pub symbol: String,
  pub current_eps: f64,
  pub quarterly_eps: f64,
  pub historical_eps: Vec<f64>,
  pub quarterly_data: Vec<QuarterlyEPSData>, // Real quarterly progression from study data
  pub price_data: Vec<PriceData>, // OHLCV price data for correlation
  pub earnings_per_share_basic_ttm: f64,
  pub market_cap_basic: f64,
  pub price_current: f64,
  pub volume: f64,
  pub sector: String,
  pub country: String,
  pub company_name: String,
}
