use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::stock::screener::models::TableDataMetrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct StockData {
    pub fetch_date: String,
    pub stocks: Vec<TableDataMetrics>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StockData {
    pub fn new(stocks: Vec<TableDataMetrics>) -> Self {
        let now = Utc::now();
        Self {
            fetch_date: now.format("%Y-%m-%d").to_string(),
            stocks,
            created_at: now,
            updated_at: now,
        }
    }
}
