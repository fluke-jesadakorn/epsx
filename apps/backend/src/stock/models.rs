use serde::{ Deserialize, Serialize };
use chrono::{ DateTime, TimeZone, Utc };

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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct PhaseInfo {
    pub date: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct PhaseStatus {
    pub date: String,
    #[serde(rename = "type")]
    pub phase_type: PhaseType,
    pub active: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseType {
    Monitor,
}

#[derive(Debug, Serialize)]
pub struct ActionStatus {
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct ActionType {
    #[serde(rename = "type")]
    pub action_type: String,
    pub active: bool,
}

impl TableDataMetrics {
    pub fn format_large_number(num: f64) -> String {
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

    pub fn format_number(num: Option<f64>) -> String {
        match num {
            Some(n) if !n.is_nan() => format!("{:.2}", n),
            _ => "N/A".to_string(),
        }
    }

    pub fn format_date(timestamp: Option<i64>) -> String {
        timestamp
            .map(|ts| {
                Utc.timestamp_opt(ts, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or("N/A".to_string())
            })
            .unwrap_or("N/A".to_string())
    }

    pub fn get_analysis_phases(last_analysis: i64, next_analysis: i64) -> (PhaseInfo, PhaseStatus) {
        let now = Utc::now().timestamp() * 1000; // Convert to milliseconds
        let last_date = DateTime::from_timestamp(last_analysis, 0).unwrap();
        let next_date = DateTime::from_timestamp(next_analysis, 0).unwrap();

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
