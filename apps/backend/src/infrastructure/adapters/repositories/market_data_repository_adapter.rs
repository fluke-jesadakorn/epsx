use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::domain::trading_analytics::aggregates::stock_analysis::StockAnalysis;
use crate::domain::trading_analytics::value_objects::*;
use crate::dom::entities::stock::{Stock as LegacyStock};
use crate::dom::entities::market_data::StockScreeningResult;
use crate::infra::services::tradingview::TradingViewApiService;

/// Repository adapter for market data that bridges legacy stock system with DDD Trading Analytics
pub struct MarketDataRepositoryAdapter {
    tradingview_service: Arc<TradingViewApiService>,
}

impl MarketDataRepositoryAdapter {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self { tradingview_service }
    }

    /// Convert legacy Stock to DDD StockAnalysis
    fn convert_legacy_to_ddd_stock(&self, legacy_stock: &LegacyStock) -> Result<StockAnalysis, String> {
        let symbol = StockSymbol::new(legacy_stock.sym().value().to_string())
            .map_err(|e| format!("Invalid stock symbol: {}", e))?;
        
        // Create basic EPS values (unknown from legacy stock)
        let current_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid current EPS: {}", e))?;
        let previous_eps = EPSValue::new(0.0)
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        // Create basic stock analysis from legacy stock data
        let stock_analysis = StockAnalysis::new(
            symbol,
            "Unknown Company".to_string(), // Legacy stock doesn't have company name
            current_eps,
            previous_eps,
            MarketSector::new("Unknown".to_string()).unwrap(),
            Country::new("unknown".to_string()).unwrap(),
        );

        stock_analysis
    }

    /// Convert StockScreeningResult to DDD StockAnalysis
    fn convert_screening_result_to_ddd(&self, screening_result: &StockScreeningResult) -> Result<StockAnalysis, String> {
        let symbol = StockSymbol::new(screening_result.symbol.clone())
            .map_err(|e| format!("Invalid symbol: {}", e))?;

        // Parse EPS from current_metric or use 0.0 as fallback
        let current_eps_value = screening_result.current_metric.parse::<f64>().unwrap_or(0.0);
        let current_eps = EPSValue::new(current_eps_value)
            .map_err(|e| format!("Invalid current EPS: {}", e))?;

        // Calculate previous EPS from growth rate
        let growth_rate = screening_result.growth_rate.parse::<f64>().unwrap_or(0.0);
        let previous_eps_value = if growth_rate != 0.0 {
            current_eps_value / (1.0 + growth_rate / 100.0)
        } else {
            current_eps_value
        };
        let previous_eps = EPSValue::new(previous_eps_value.max(0.0))
            .map_err(|e| format!("Invalid previous EPS: {}", e))?;

        let sector = MarketSector::new(screening_result.sector.clone())
            .map_err(|e| format!("Invalid sector: {}", e))?;

        let country = Country::new(screening_result.country.clone())
            .map_err(|e| format!("Invalid country: {}", e))?;

        // Create stock analysis
        let stock_analysis = StockAnalysis::new(
            symbol,
            screening_result.name.clone(),
            current_eps,
            previous_eps,
            sector,
            country,
        );

        stock_analysis
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
    use crate::dom::entities::market_data::StockScreeningResult;
    use crate::dom::values::{Symbol, Market};
    use rust_decimal_macros::dec;

    #[test]
    fn test_screening_result_to_ddd_conversion() {
        let screening_result = StockScreeningResult {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_metric: "1.52".to_string(),
            growth_rate: "15.2".to_string(),
            value_index: "150.25".to_string(),
            activity_score: "45678900".to_string(),
            market_size: "2500000000000".to_string(),
            next_analysis_date: "2024-01-15".to_string(),
            last_analysis_date: "2023-10-15".to_string(),
        };

        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(Default::default())));
        let adapter = MarketDataRepositoryAdapter::new(tradingview_service);

        match adapter.convert_screening_result_to_ddd(&screening_result) {
            Ok(stock_analysis) => {
                assert_eq!(stock_analysis.symbol().as_str(), "AAPL");
                assert_eq!(stock_analysis.company_name(), "Apple Inc.");
                assert!(stock_analysis.analytics_metrics().is_some());
                
                let metrics = stock_analysis.analytics_metrics().unwrap();
                assert_eq!(metrics.eps_value.value(), 1.52);
                assert_eq!(metrics.growth_factor.percentage(), 15.2);
            }
            Err(e) => panic!("Conversion failed: {}", e),
        }
    }

    #[test]
    fn test_legacy_stock_to_ddd_conversion() {
        use crate::dom::values::{Symbol, Market};
        use rust_decimal_macros::dec;

        let symbol = Symbol::new("AAPL").unwrap();
        let legacy_stock = LegacyStock::new(symbol, dec!(150.50), 1000000, Market::NASDAQ);

        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(Default::default())));
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