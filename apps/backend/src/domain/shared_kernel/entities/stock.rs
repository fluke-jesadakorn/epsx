// Stock entity for shared use across bounded contexts

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Stock entity representing a tradeable asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub symbol: String,
    pub name: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub volume: Option<u64>,
    pub updated_at: DateTime<Utc>,
}

impl Stock {
    pub fn new(symbol: String, name: String) -> Self {
        Self {
            symbol,
            name,
            sector: None,
            industry: None,
            market_cap: None,
            price: None,
            volume: None,
            updated_at: Utc::now(),
        }
    }

    pub fn with_market_data(mut self, price: f64, volume: u64, market_cap: Option<f64>) -> Self {
        self.price = Some(price);
        self.volume = Some(volume);
        self.market_cap = market_cap;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_classification(mut self, sector: String, industry: String) -> Self {
        self.sector = Some(sector);
        self.industry = Some(industry);
        self
    }
}