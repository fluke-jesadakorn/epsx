// Market data entities for shared use

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Stock screening result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockScreeningResult {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_percent: f64,
    pub volume: u64,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub sector: Option<String>,
    pub meets_criteria: bool,
    pub score: f64,
    pub screened_at: DateTime<Utc>,
}

impl StockScreeningResult {
    pub fn new(symbol: String, name: String, price: f64) -> Self {
        Self {
            symbol,
            name,
            price,
            change_percent: 0.0,
            volume: 0,
            market_cap: None,
            pe_ratio: None,
            sector: None,
            meets_criteria: false,
            score: 0.0,
            screened_at: Utc::now(),
        }
    }

    pub fn with_criteria_result(mut self, meets_criteria: bool, score: f64) -> Self {
        self.meets_criteria = meets_criteria;
        self.score = score;
        self.screened_at = Utc::now();
        self
    }
}