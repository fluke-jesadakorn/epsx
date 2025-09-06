
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// EPS (Earnings Per Share) ranking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSRanking {
    pub symbol: String,
    pub company_name: String,
    pub eps_current: f64,
    pub eps_previous: f64,
    pub growth_rate: f64,
    pub rank: u32,
    pub sector: String,
    pub market_cap: Option<f64>,
    pub last_updated: DateTime<Utc>,
}

impl EPSRanking {
    pub fn new(
        symbol: String,
        company_name: String,
        eps_current: f64,
        eps_previous: f64,
        sector: String,
    ) -> Self {
        let growth_rate = if eps_previous != 0.0 {
            ((eps_current - eps_previous) / eps_previous) * 100.0
        } else {
            0.0
        };

        Self {
            symbol,
            company_name,
            eps_current,
            eps_previous,
            growth_rate,
            rank: 0,
            sector,
            market_cap: None,
            last_updated: Utc::now(),
        }
    }

    pub fn with_rank(mut self, rank: u32) -> Self {
        self.rank = rank;
        self
    }

    pub fn with_market_cap(mut self, market_cap: f64) -> Self {
        self.market_cap = Some(market_cap);
        self
    }

    pub fn growth_percentage(&self) -> f64 {
        self.growth_rate
    }

    pub fn is_positive_growth(&self) -> bool {
        self.growth_rate > 0.0
    }
}

/// Comprehensive EPS growth data for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSGrowthData {
    pub symbol: String,
    pub company_name: String,
    pub quarterly_eps: Vec<f64>,
    pub annual_eps: Vec<f64>,
    pub quarterly_growth_rates: Vec<f64>,
    pub annual_growth_rates: Vec<f64>,
    pub average_quarterly_growth: f64,
    pub average_annual_growth: f64,
    pub volatility: f64,
    pub trend: GrowthTrend,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthTrend {
    Accelerating,
    Steady,
    Decelerating,
    Volatile,
    Declining,
}

impl EPSGrowthData {
    pub fn new(symbol: String, company_name: String) -> Self {
        Self {
            symbol,
            company_name,
            quarterly_eps: Vec::new(),
            annual_eps: Vec::new(),
            quarterly_growth_rates: Vec::new(),
            annual_growth_rates: Vec::new(),
            average_quarterly_growth: 0.0,
            average_annual_growth: 0.0,
            volatility: 0.0,
            trend: GrowthTrend::Steady,
            last_updated: Utc::now(),
        }
    }
}

/// Pagination information for EPS data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSPagination {
    pub page: i32,
    pub per_page: i32,
    pub total: i64,
}

impl EPSPagination {
    pub fn new(page: i32, per_page: i32, total: i64) -> Self {
        Self {
            page,
            per_page,
            total,
        }
    }
}