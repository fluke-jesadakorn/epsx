
use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::domain::market_analytics::aggregates::stock_analysis::StockAnalysis;
use crate::domain::market_analytics::value_objects::*;
use crate::domain::shared_kernel::entities::stock::{Stock as LegacyStock};
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;

/// Repository adapter for market data that bridges legacy stock system with DDD Trading Analytics
#[derive(Clone)]
pub struct MarketDataRepositoryAdapter {
    #[allow(dead_code)]
    tradingview_service: Arc<TradingViewApiService>,
}


impl MarketDataRepositoryAdapter {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }

    /// Convert legacy Stock to DDD StockAnalysis
    fn convert_legacy_to_ddd_stock(&self, legacy_stock: &LegacyStock) -> Result<StockAnalysis, String> {
        let symbol = StockSymbol::new(legacy_stock.symbol.clone())
            .map_err(|e| format!("Invalid stock symbol: {}", e))?;
        
        // Create basic EPS values (unknown from legacy stock)
        let current_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid current EPS: {}", e))?;
        let previous_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        // Create basic stock analysis from legacy stock data
        StockAnalysis::new(
            symbol,
            "Unknown Company".to_string(), // Legacy stock doesn't have company name
            current_eps,
            previous_eps,
            MarketSector::new("Unknown".to_string()).unwrap(),
            Country::new("unknown".to_string()).unwrap(),
        )
    }

    /// Convert StockScreeningResult to DDD StockAnalysis
    fn convert_screening_result_to_ddd(&self, screening_result: &StockScreeningResult) -> Result<StockAnalysis, String> {
        let symbol = StockSymbol::new(screening_result.symbol.clone())
            .map_err(|e| format!("Invalid symbol: {}", e))?;

        // Parse EPS from current_metric or use 0.0 as fallback
        let current_eps_value = if let Some(pe_ratio) = screening_result.pe_ratio {
            screening_result.price / pe_ratio.max(1.0)  // Calculate EPS from price and P/E ratio
        } else {
            1.0  // Default EPS value
        };
        let current_eps = EPSValue::new(current_eps_value)
            .map_err(|e| format!("Invalid current EPS: {}", e))?;

        // Calculate previous EPS from growth rate
        let growth_rate = screening_result.change_percent; // Use change_percent as growth proxy
        let previous_eps_value = if growth_rate != 0.0 {
            current_eps_value / (1.0 + growth_rate / 100.0)
        } else {
            current_eps_value
        };
        let previous_eps = EPSValue::new(previous_eps_value.max(0.0))
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        let sector = MarketSector::new(screening_result.sector.clone().unwrap_or_else(|| "Unknown".to_string()))
            .map_err(|e| format!("Invalid sector: {}", e))?;

        let country = Country::new("US".to_string()) // Default country as field not available
            .map_err(|e| format!("Invalid default country: {}", e))?;

        // Create stock analysis
        StockAnalysis::new(
            symbol,
            screening_result.name.clone(),
            current_eps,
            previous_eps,
            sector,
            country,
        )
    }

}

impl MarketDataRepositoryAdapter {
    /// Convert screening result to stock analysis
    pub fn screening_result_to_stock_analysis(&self, screening_result: &StockScreeningResult) -> Result<StockAnalysis, String> {
        self.convert_screening_result_to_ddd(screening_result)
    }

    /// Convert legacy stock to stock analysis
    pub fn legacy_stock_to_stock_analysis(&self, legacy_stock: &LegacyStock) -> Result<StockAnalysis, String> {
        self.convert_legacy_to_ddd_stock(legacy_stock)
    }
}

/// Criteria for finding stocks in market data
#[derive(Debug, Clone)]
pub struct MarketDataCriteria {
    pub sector_filter: Option<SectorCategory>,
    pub country_filter: Option<Country>,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
    pub min_market_cap: Option<u64>,
    pub limit: Option<usize>,
}

/// Market statistics summary
#[derive(Debug, Clone)]
pub struct MarketStatistics {
    pub total_stocks: u64,
    pub avg_eps: f64,
    pub avg_growth: f64,
    pub total_market_cap: u64,
    pub calculated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn get_test_config() -> crate::config::Config {
        crate::config::get_fallback_config()
    }
    use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;

    #[test]
    fn test_screening_result_to_ddd_conversion() {
        let screening_result = StockScreeningResult {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            price: 150.0,
            change_percent: 0.0,
            volume: 50000000,
            market_cap: Some(2500000000.0),
            pe_ratio: None,
            sector: Some("Technology".to_string()),
            meets_criteria: true,
            score: 85.5,
            screened_at: Utc::now(),
            current_eps: Some(1.52),
            eps_growth_yoy: Some(15.2),
            earnings_forecast_fq: Some(1.60),
            earnings_forecast_next_fq: Some(1.65),
            eps_q_minus_2: Some(1.40),
            eps_q_minus_1: Some(1.48),
            eps_q_current: Some(1.52),
            eps_q_next_estimate: Some(1.60),
            eps_q_minus_2_date: Some("2023-07-15".to_string()),
            eps_q_minus_1_date: Some("2023-10-15".to_string()),
            eps_q_current_date: Some("2024-01-15".to_string()),
            eps_q_next_estimate_date: Some("2024-04-15".to_string()),
            qoq_growth_current: Some(2.7),
            yoy_growth_current: Some(15.2),
            trend_direction: Some("UP".to_string()),
            avg_growth_rate: Some(12.5),
            consistency_score: Some("HIGH".to_string()),
            next_earnings_date: None,
            last_earnings_date: None,
            currency: Some("USD".to_string()),
        };

        let config = get_test_config();
        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let adapter = MarketDataRepositoryAdapter::new(tradingview_service);

        match adapter.convert_screening_result_to_ddd(&screening_result) {
            Ok(stock_analysis) => {
                assert_eq!(stock_analysis.symbol().as_str(), "AAPL");
                assert_eq!(stock_analysis.company_name(), "Apple Inc.");
                assert_eq!(stock_analysis.current_eps().value(), 1.52);
                assert_eq!(stock_analysis.eps_growth().percentage(), 15.2);
            }
            Err(e) => panic!("Conversion failed: {}", e),
        }
    }

    #[test]
    fn test_legacy_stock_to_ddd_conversion() {
        use crate::domain::shared_kernel::value_objects::Symbol;

        let symbol = Symbol::new("AAPL").unwrap();
        let legacy_stock = LegacyStock::new(symbol.to_string(), "Apple Inc.".to_string());

        let config = get_test_config();
        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let adapter = MarketDataRepositoryAdapter::new(tradingview_service);

        match adapter.convert_legacy_to_ddd_stock(&legacy_stock) {
            Ok(stock_analysis) => {
                assert_eq!(stock_analysis.symbol().as_str(), "AAPL");
                assert_eq!(stock_analysis.company_name(), "Unknown Company");
                // Basic EPS values are set to 0.0 by default
                assert_eq!(stock_analysis.current_eps().value(), 0.0);
                assert_eq!(stock_analysis.previous_eps().value(), 0.0);
            }
            Err(e) => panic!("Conversion failed: {}", e),
        }
    }

    #[test]
    fn test_market_data_criteria() {
        let criteria = MarketDataCriteria {
            sector_filter: Some(SectorCategory::Technology),
            country_filter: Some(Country::new("america".to_string()).unwrap()),
            min_eps: Some(1.0),
            min_growth: Some(10.0),
            min_market_cap: Some(1000000000),
            limit: Some(50),
        };

        assert!(criteria.sector_filter.is_some());
        assert!(criteria.country_filter.is_some());
        assert_eq!(criteria.min_eps, Some(1.0));
        assert_eq!(criteria.limit, Some(50));
    }
}