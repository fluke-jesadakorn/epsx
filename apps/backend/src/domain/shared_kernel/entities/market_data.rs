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
    // Real EPS data from TradingView - Legacy fields (kept for compatibility)
    pub current_eps: Option<f64>,           // earnings_per_share_fq or ttm
    pub eps_growth_yoy: Option<f64>,        // earnings_per_share_diluted_yoy_growth_ttm
    pub earnings_forecast_fq: Option<f64>,  // earnings_per_share_forecast_fq
    pub earnings_forecast_next_fq: Option<f64>, // earnings_per_share_forecast_next_fq
    
    // 4-Quarter EPS Data Structure
    pub eps_q_minus_2: Option<f64>,        // Q-2 (2 quarters ago)
    pub eps_q_minus_1: Option<f64>,        // Q-1 (1 quarter ago) 
    pub eps_q_current: Option<f64>,        // Q0 (current quarter) - same as current_eps
    pub eps_q_next_estimate: Option<f64>,  // Q+1 (next quarter estimate)
    
    // Quarter dates for EPS reporting
    pub eps_q_minus_2_date: Option<String>,
    pub eps_q_minus_1_date: Option<String>, 
    pub eps_q_current_date: Option<String>,
    pub eps_q_next_estimate_date: Option<String>,
    
    // Growth calculations
    pub qoq_growth_current: Option<f64>,   // Q0 vs Q-1 growth percentage
    pub yoy_growth_current: Option<f64>,   // Q0 vs Q-4 growth (if available)
    pub trend_direction: Option<String>,   // "UP", "DOWN", "FLAT"
    pub avg_growth_rate: Option<f64>,      // Average growth rate across available quarters
    pub consistency_score: Option<String>, // "HIGH", "MEDIUM", "LOW" - earnings consistency
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
            current_eps: None,
            eps_growth_yoy: None,
            earnings_forecast_fq: None,
            earnings_forecast_next_fq: None,
            // Initialize 4-quarter EPS fields
            eps_q_minus_2: None,
            eps_q_minus_1: None,
            eps_q_current: None,
            eps_q_next_estimate: None,
            // Initialize quarter dates
            eps_q_minus_2_date: None,
            eps_q_minus_1_date: None,
            eps_q_current_date: None,
            eps_q_next_estimate_date: None,
            // Initialize growth calculations
            qoq_growth_current: None,
            yoy_growth_current: None,
            trend_direction: None,
            avg_growth_rate: None,
            consistency_score: None,
        }
    }

    pub fn with_criteria_result(mut self, meets_criteria: bool, score: f64) -> Self {
        self.meets_criteria = meets_criteria;
        self.score = score;
        self.screened_at = Utc::now();
        self
    }

    /// Calculate quarterly growth metrics and trend analysis
    pub fn with_quarterly_analysis(mut self) -> Self {
        // Calculate QoQ growth (Q0 vs Q-1)
        if let (Some(current), Some(previous)) = (self.eps_q_current, self.eps_q_minus_1) {
            if previous != 0.0 {
                self.qoq_growth_current = Some(((current - previous) / previous) * 100.0);
            }
        }

        // Calculate average growth rate across available quarters
        let mut growth_rates = Vec::new();
        if let Some(qoq) = self.qoq_growth_current {
            growth_rates.push(qoq);
        }
        if let (Some(q1), Some(q2)) = (self.eps_q_minus_1, self.eps_q_minus_2) {
            if q2 != 0.0 {
                growth_rates.push(((q1 - q2) / q2) * 100.0);
            }
        }
        if !growth_rates.is_empty() {
            self.avg_growth_rate = Some(growth_rates.iter().sum::<f64>() / growth_rates.len() as f64);
        }

        // Determine trend direction
        if let Some(avg_growth) = self.avg_growth_rate {
            self.trend_direction = Some(if avg_growth > 5.0 {
                "UP".to_string()
            } else if avg_growth < -5.0 {
                "DOWN".to_string()
            } else {
                "FLAT".to_string()
            });
        }

        // Calculate consistency score
        if growth_rates.len() >= 2 {
            let variance = growth_rates.iter()
                .map(|x| (x - self.avg_growth_rate.unwrap_or(0.0)).powi(2))
                .sum::<f64>() / growth_rates.len() as f64;
            let std_dev = variance.sqrt();
            
            self.consistency_score = Some(if std_dev < 10.0 {
                "HIGH".to_string()
            } else if std_dev < 25.0 {
                "MEDIUM".to_string()
            } else {
                "LOW".to_string()
            });
        }

        // Sync with legacy field for backward compatibility
        if self.current_eps.is_none() && self.eps_q_current.is_some() {
            self.current_eps = self.eps_q_current;
        }

        self
    }
}