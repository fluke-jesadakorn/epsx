use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, warn};
use epsx_contracts::value_objects::QuarterlyEPSData;

/// EPS Growth data entity for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPSGrowthData {
    pub symbol: String,
    pub name: String,
    pub country: String,
    pub sector: String,
    pub exchange: String,
    pub current_eps: Option<f64>,
    pub growth_factor: Option<f64>,
    pub price_current: Option<f64>,
    pub market_cap: Option<i64>,
    pub volume: Option<i64>,
    pub ranking_score: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    // Real earnings announcement dates from TradingView
    pub next_earnings_date: Option<String>,
    pub last_earnings_date: Option<String>,
}

/// EPS Ranking result for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EPSRanking {
    pub symbol: String,
    pub name: String,
    pub country: String,
    pub sector: String,
    pub exchange: String,
    pub current_eps: Option<f64>,
    pub growth_factor: Option<f64>,
    pub price_current: Option<f64>,
    pub market_cap: Option<i64>,
    pub volume: Option<i64>,
    pub ranking_position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarterly_data: Option<Vec<QuarterlyEPSData>>,
    // Real earnings announcement dates from TradingView
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_earnings_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_earnings_date: Option<String>,
}

/// API response structure for EPS rankings
#[derive(Debug, Serialize, Deserialize)]
pub struct EPSRankingsResponse {
    pub rankings: Vec<EPSRanking>,
    pub pagination: EPSPagination,
}

/// Pagination structure matching frontend pattern
#[derive(Debug, Serialize, Deserialize)]
pub struct EPSPagination {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

/// EPS Growth trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EPSGrowthTrend {
    Accelerating,
    Steady,
    Decelerating,
    Volatile,
    Unknown,
}

/// Parameters for creating EPSGrowthData
pub struct EPSGrowthParams {
    pub symbol: String,
    pub name: String,
    pub country: String,
    pub sector: String,
    pub exchange: String,
    pub current_eps: Option<f64>,
    pub growth_factor: Option<f64>,
    pub price_current: Option<f64>,
    pub market_cap: Option<i64>,
    pub volume: Option<i64>,
}

impl EPSGrowthData {
    /// Create new EPS growth data with validation
    pub fn new(params: EPSGrowthParams) -> Self {
        debug!("Creating new EPS growth data for symbol: {}", params.symbol);

        Self {
            symbol: params.symbol,
            name: params.name,
            country: params.country,
            sector: params.sector,
            exchange: params.exchange,
            current_eps: params.current_eps,
            growth_factor: params.growth_factor,
            price_current: params.price_current,
            market_cap: params.market_cap,
            volume: params.volume,
            ranking_score: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            next_earnings_date: None, // Will be set when converting from StockScreeningResult
            last_earnings_date: None, // Will be set when converting from StockScreeningResult
        }
    }

    /// Validate EPS data quality
    pub fn validate(&self) -> Result<(), String> {
        debug!("Validating EPS data for symbol: {}", self.symbol);

        if self.symbol.is_empty() {
            warn!("EPS validation failed: empty symbol");
            return Err("Symbol cannot be empty".to_string());
        }

        if self.country.is_empty() {
            warn!("EPS validation failed: empty country for symbol {}", self.symbol);
            return Err("Country cannot be empty".to_string());
        }

        // Validate EPS values are reasonable
        if let Some(eps) = self.current_eps {
            if !(-1000.0..=1000.0).contains(&eps) {
                warn!("EPS validation warning: unusual EPS value {} for symbol {}", eps, self.symbol);
            }
        }

        // Validate QoQ growth is reasonable
        if let Some(growth) = self.growth_factor {
            if !(-500.0..=1000.0).contains(&growth) {
                warn!("EPS validation warning: extreme QoQ growth {} for symbol {}", growth, self.symbol);
            }
        }

        debug!("EPS validation passed for symbol: {}", self.symbol);
        Ok(())
    }

    /// Calculate ranking score based on multiple factors
    pub fn calculate_ranking_score(&mut self) {
        debug!("Calculating ranking score for symbol: {}", self.symbol);

        let mut score = 0.0;

        // QoQ growth weight (40%)
        if let Some(qoq) = self.growth_factor {
            score += qoq * 0.4;
        }

        // Market cap weight (20%) - larger companies get slight bonus
        if let Some(mc) = self.market_cap {
            let mc_score = (mc as f64 / 1_000_000_000.0).ln().max(0.0); // Log scale bonus
            score += mc_score * 0.2;
        }

        // Volume weight (10%) - higher volume gets bonus
        if let Some(vol) = self.volume {
            let vol_score = (vol as f64 / 1_000_000.0).ln().max(0.0); // Log scale bonus
            score += vol_score * 0.1;
        }

        // EPS absolute value weight (30%)
        if let Some(eps) = self.current_eps {
            if eps > 0.0 {
                score += eps.ln().max(0.0) * 0.3;
            }
        }

        self.ranking_score = Some(score);
        debug!("Calculated ranking score: {} for symbol: {}", score, self.symbol);
    }

    /// Determine EPS growth trend
    pub fn get_growth_trend(&self) -> EPSGrowthTrend {
        match self.growth_factor {
            Some(growth) if growth > 50.0 => EPSGrowthTrend::Accelerating,
            Some(growth) if growth >= 10.0 => EPSGrowthTrend::Steady,
            Some(growth) if growth >= 0.0 => EPSGrowthTrend::Steady,
            Some(growth) if growth >= -10.0 => EPSGrowthTrend::Decelerating,
            Some(_) => EPSGrowthTrend::Volatile,
            None => EPSGrowthTrend::Unknown,
        }
    }

    /// Check if this stock has quality EPS data
    pub fn has_quality_data(&self) -> bool {
        let has_eps = self.current_eps.is_some();
        let has_growth = self.growth_factor.is_some();
        let has_price = self.price_current.is_some();
        
        debug!("Quality check for {}: EPS={}, Growth={}, Price={}", 
               self.symbol, has_eps, has_growth, has_price);
        
        has_eps && has_growth && has_price
    }
}

impl EPSRanking {
    /// Convert from EPS growth data to ranking
    pub fn from_eps_data(data: EPSGrowthData, position: Option<i32>) -> Self {
        Self {
            symbol: data.symbol,
            name: data.name,
            country: data.country,
            sector: data.sector,
            exchange: data.exchange,
            current_eps: data.current_eps,
            growth_factor: data.growth_factor,
            price_current: data.price_current,
            market_cap: data.market_cap,
            volume: data.volume,
            ranking_position: position,
            quarterly_data: None, // Will be populated by WebSocket enhancement
            next_earnings_date: data.next_earnings_date,
            last_earnings_date: data.last_earnings_date,
        }
    }
}


impl EPSPagination {
    /// Create pagination info
    pub fn new(page: i32, limit: i32, total: i64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as i32;
        
        Self {
            page,
            limit,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eps_data_validation() {
        let eps_data = EPSGrowthData::new(EPSGrowthParams {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.52),
            growth_factor: Some(15.2),
            price_current: Some(150.25),
            market_cap: Some(2500000000000),
            volume: Some(45678900),
        });

        assert!(eps_data.validate().is_ok());
        assert!(eps_data.has_quality_data());
    }

    #[test]
    fn test_ranking_score_calculation() {
        let mut eps_data = EPSGrowthData::new(EPSGrowthParams {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.52),
            growth_factor: Some(15.2),
            price_current: Some(150.25),
            market_cap: Some(2500000000000),
            volume: Some(45678900),
        });

        eps_data.calculate_ranking_score();
        assert!(eps_data.ranking_score.is_some());
        assert!(eps_data.ranking_score.unwrap() > 0.0);
    }

    #[test]
    fn test_pagination_calculation() {
        let pagination = EPSPagination::new(1, 50, 1547);
        assert_eq!(pagination.total_pages, 31);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);
    }
}