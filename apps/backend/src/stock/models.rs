// Stock screening and financial models
use serde::{Deserialize, Serialize};
use chrono::{TimeZone, Utc};
use utoipa::ToSchema;

/// Table data metrics for stock screening results
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TableDataMetrics {
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

/// WebSocket quote session creation message
#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteSessionCreate {
    pub session: String,
}

impl QuoteSessionCreate {
    pub fn new(session_id: &str) -> Self {
        Self {
            session: session_id.to_string(),
        }
    }
}

/// Number formatting trait for financial data
pub trait NumberFormatter {
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

impl NumberFormatter for TableDataMetrics {}

impl TableDataMetrics {
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

/// TradingView response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewResponse {
    #[serde(default)]
    pub total_count: i32,
    pub data: Vec<TradingViewStock>,
}

/// Individual stock data from TradingView
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewStock {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "d")]
    pub data: Vec<StockDataField>,
}

/// Stock data field (can be string or number)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StockDataField {
    String(String),
    Number(f64),
}

/// EPS growth ranking query parameters
#[derive(Debug, Deserialize, ToSchema)]
pub struct EpsGrowthRankingParams {
    pub limit: Option<i32>,
    pub skip: Option<i32>,
    pub sort_by: Option<String>,
    // Enhanced filtering support
    pub country: Option<String>,
    pub sector: Option<String>,
}