use serde::{Deserialize, Serialize};
use chrono::{TimeZone, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewResponse {
    #[serde(default)]
    pub total_count: i32,
    pub data: Vec<TradingViewStock>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingViewStock {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "d")]
    pub data: Vec<StockDataField>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StockDataField {
    String(String),
    Number(f64),
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PhaseInfo {
    pub date: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PhaseStatus {
    pub date: String,
    #[serde(rename = "type")]
    pub phase_type: PhaseType,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PhaseType {
    Monitor,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ActionStatus {
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ActionType {
    #[serde(rename = "type")]
    pub action_type: String,
    pub active: bool,
}

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
