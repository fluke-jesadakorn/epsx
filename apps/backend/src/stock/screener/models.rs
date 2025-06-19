use serde::{Deserialize, Serialize};
use crate::stock::common::{NumberFormatter, PhaseInfo, PhaseType, PhaseStatus, ActionStatus, ActionType};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteSessionCreate {
    pub m: String,
    pub p: Vec<String>,
}

impl QuoteSessionCreate {
    pub fn new(session_id: &str) -> Self {
        Self {
            m: "quote_create_session".to_string(),
            p: vec![session_id.to_string()],
        }
    }
}

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
    pub entry_phase: PhaseInfo,
    pub phase_status: PhaseStatus,
    pub metric_score: String,
    pub growth_indicator: String,
    pub current_metric: String,
    pub predicted_metric: String,
    pub last_analysis_date: String,
    pub next_analysis_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_buy: Option<ActionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_action: Option<ActionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eps_growth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_earnings_date: Option<String>,
}

impl NumberFormatter for TableDataMetrics {}

impl TableDataMetrics {
    pub fn get_analysis_phases(last_analysis: i64, next_analysis: i64) -> (PhaseInfo, PhaseStatus) {
        let now = chrono::Utc::now().timestamp() * 1000; // Convert to milliseconds
        let last_date = chrono::DateTime::from_timestamp(last_analysis, 0).unwrap();
        let next_date = chrono::DateTime::from_timestamp(next_analysis, 0).unwrap();

        // Calculate entry date (1 day after last analysis)
        let entry_date = last_date + chrono::Duration::days(1);

        // Calculate monitor date (7 days before next analysis)
        let monitor_date = next_date - chrono::Duration::days(7);

        (
            PhaseInfo {
                date: entry_date.format("%Y-%m-%d").to_string(),
                active: now >= entry_date.timestamp_millis() &&
                now < monitor_date.timestamp_millis(),
            },
            PhaseStatus {
                date: monitor_date.format("%Y-%m-%d").to_string(),
                phase_type: PhaseType::Monitor,
                active: now >= monitor_date.timestamp_millis(),
            },
        )
    }
}

// Helper function to parse formatted numbers (e.g., "1.23M" -> 1230000.0)
pub fn parse_formatted_number(formatted: &str) -> f64 {
    let cleaned = formatted.replace(['T', 'B', 'M'], "");
    let base = cleaned.parse::<f64>().unwrap_or(0.0);
    
    if formatted.contains('T') {
        base * 1e12
    } else if formatted.contains('B') {
        base * 1e9
    } else if formatted.contains('M') {
        base * 1e6
    } else {
        base
    }
}
