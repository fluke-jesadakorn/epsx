// TradingView Mapper - Focused Module for Data Transformation and Mapping
use uuid::Uuid;
// Handles conversion between TradingView data and internal/frontend formats

use tracing::{debug, error, info, warn};


use crate::dom::entities::market_data::{TradingViewStock, StockDataField};

use crate::dom::entities::eps_growth::EPSGrowthData;

use super::types::{FrontendEPSData, StockDataResult};


/// Data mapper for TradingView API responses
pub struct TradingViewMapper;

impl TradingViewMapper {
    /// Convert TradingView stock data to EPS growth data
    pub fn convert_to_eps_growth_data(stock: TradingViewStock) -> StockDataResult<EPSGrowthData> {
        debug!("Converting TradingView stock to EPS data: {}", stock.s);

        let get_number = |data: &[StockDataField], idx: usize| -> Option<f64> {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => Some(*n),
                Some(StockDataField::Integer(i)) => Some(*i as f64),
                _ => None
            }
        };

        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            data.get(idx)
                .map(|field| match field {
                    StockDataField::String(s) => s.clone(),
                    StockDataField::Number(n) => n.to_string(),
                    StockDataField::Integer(i) => i.to_string(),
                    StockDataField::Boolean(b) => b.to_string(),
                    StockDataField::Array(_) => "Array".to_string(),
                    StockDataField::Object(_) => "Object".to_string(),
                    StockDataField::Null => default.to_string(),
                })
                .unwrap_or_else(|| default.to_string())
        };

        // Extract symbol from full symbol (e.g., "NASDAQ:AAPL" -> "AAPL")
        let symbol = stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string();
        let name = get_string(&stock.d, 0, ""); // name
        let country = get_string(&stock.d, 25, "unknown"); // market (index 25, shifted by +2)
        let sector = get_string(&stock.d, 24, ""); // sector.tr (index 24, shifted by +2)
        let exchange = get_string(&stock.d, 29, ""); // exchange (index 29, shifted by +2)
        
        // EPS data extraction using dynamic detection algorithm
        let current_eps = Self::detect_quarterly_eps_dynamically(&stock.d, &symbol);
        
        // QoQ growth will be calculated dynamically from quarterly data, not from TTM fields
        let qoq_growth = None; // Remove TTM-based growth calculation
        let price_current = get_number(&stock.d, 6); // close
        let market_cap = get_number(&stock.d, 16).map(|mc| mc as i64); // market_cap_basic (index 16)
        let volume = get_number(&stock.d, 13).map(|vol| vol as i64); // volume

        debug!("Extracted EPS data for {}: EPS={:?}, QoQ Growth={:?}, Price={:?}", 
               symbol, current_eps, qoq_growth, price_current);

        // Validate essential data
        if symbol.is_empty() {
            warn!("Empty symbol for TradingView stock: {:?}", stock.s);
            return Err("Empty symbol".to_string());
        }

        if country.is_empty() || country == "unknown" {
            warn!("Missing country for symbol: {}", symbol);
        }

        let eps_data = EPSGrowthData::new(
            symbol.clone(),
            name,
            country.to_lowercase(),
            sector,
            exchange,
            current_eps,
            qoq_growth,
            price_current,
            market_cap,
            volume,
        );

        // Validate the created data
        eps_data.validate().map_err(|e| {
            error!("EPS data validation failed for {}: {}", symbol, e);
            e
        })?;

        debug!("Successfully converted TradingView data to EPS data for: {}", symbol);
        Ok(eps_data)
    }

    /// Dynamically detect the correct quarterly EPS field without hardcoded logic
    fn detect_quarterly_eps_dynamically(data: &[StockDataField], symbol: &str) -> Option<f64> {
        let get_number = |data: &[StockDataField], idx: usize| -> Option<f64> {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => Some(*n),
                Some(StockDataField::Integer(i)) => Some(*i as f64),
                _ => None
            }
        };

        // Candidate quarterly EPS fields only - NO TTM fields per user request
        let candidates = vec![
            (14, "earnings_per_share_fq", "quarterly"),           // Primary quarterly EPS field
            (22, "earnings_per_share_forecast_fq", "quarterly"),   // Forecast quarterly
            (23, "earnings_per_share_forecast_next_fq", "quarterly"), // Next quarter forecast
        ];

        let mut field_values = Vec::new();
        for (idx, name, field_type) in candidates {
            if let Some(value) = get_number(data, idx) {
                field_values.push((idx, name, field_type, value));
            }
        }

        debug!("EPS field candidates for {}: {:?}", symbol, field_values);

        // Dynamic selection algorithm
        Self::select_best_quarterly_eps_field(field_values, symbol)
    }

    /// Select the best quarterly EPS field using dynamic intelligence
    fn select_best_quarterly_eps_field(candidates: Vec<(usize, &str, &str, f64)>, symbol: &str) -> Option<f64> {
        if candidates.is_empty() {
            return None;
        }

        // Filter out clearly invalid values using dynamic validation
        let valid_candidates: Vec<_> = candidates.into_iter()
            .filter(|(_, _, _, value)| Self::is_valid_quarterly_eps_value(*value))
            .collect();

        if valid_candidates.is_empty() {
            warn!("No valid quarterly EPS candidates found for {}", symbol);
            return None;
        }

        // All candidates are quarterly fields - prioritize primary field (index 14)
        for (idx, name, _, value) in &valid_candidates {
            if *idx == 14 {
                info!("Selected primary quarterly EPS field {} for {}: {}", name, symbol, value);
                return Some(*value);
            }
        }
        
        // Fallback to first available quarterly field
        let (_idx, name, _, value) = &valid_candidates[0];
        info!("Selected quarterly EPS field {} for {}: {}", name, symbol, value);
        Some(*value)
    }

    /// Dynamic validation for quarterly EPS values - consistent with WebSocket service
    fn is_valid_quarterly_eps_value(eps: f64) -> bool {
        // Basic sanity checks
        if !eps.is_finite() || eps <= 0.0 {
            return false;
        }

        // Allow wide range for different markets and currencies
        // Same logic as WebSocket service for consistency
        if eps > super::types::constants::MAX_EPS_VALUE {
            warn!("EPS value {} seems extremely high, might be an error", eps);
            return false;
        }

        if eps < super::types::constants::MIN_EPS_VALUE {
            warn!("EPS value {} is very small, might be noise", eps);
            return false;
        }

        true
    }

    /// Map TradingView stock to frontend EPS data format
    pub fn map_to_frontend_eps_data(stock: TradingViewStock) -> FrontendEPSData {
        // Helper functions to extract data from the array
        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            match data.get(idx) {
                Some(StockDataField::String(s)) => s.clone(),
                Some(StockDataField::Number(n)) => n.to_string(),
                Some(StockDataField::Integer(i)) => i.to_string(),
                _ => default.to_string(),
            }
        };

        let get_number = |data: &[StockDataField], idx: usize, default: f64| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => default,
            }
        };

        // Extract data according to TradingView column order from your capture:
        let symbol = stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string(); // Extract symbol from "NASDAQ:NVDA"
        let company_name = get_string(&stock.d, 1, "Unknown Company"); // description
        let price_current = get_number(&stock.d, 6, 0.0); // close
        let volume = get_number(&stock.d, 13, 0.0) as i64; // volume  
        let market_cap = get_number(&stock.d, 15, 0.0) as i64; // market_cap_basic
        
        // Use quarterly EPS directly (field index 14 = earnings_per_share_fq)
        let quarterly_eps = get_number(&stock.d, 14, 0.0); // earnings_per_share_fq
        let ttm_eps = get_number(&stock.d, 19, 0.0); // earnings_per_share_diluted_ttm
        let basic_ttm = get_number(&stock.d, 18, 0.0); // earnings_per_share_basic_ttm
        
        // QoQ growth should be calculated from quarterly progression, not TTM YoY growth
        // Using None here - will be calculated from WebSocket quarterly data in enhanced mode
        let qoq_growth = None; // Remove TTM-based YoY growth calculation
        let sector = get_string(&stock.d, 24, "Unknown"); // sector.tr (shifted by +3)
        let country = get_string(&stock.d, 25, "unknown"); // market (shifted by +3)
        
        // Debug log for Taiwan stocks to diagnose EPS issue
        if country == "taiwan" || symbol == "2330" {
            Self::log_taiwan_stock_debug(&symbol, quarterly_eps, ttm_eps, basic_ttm, &stock);
        }
        
        // Use quarterly EPS directly - no TTM fallback needed
        let current_eps = quarterly_eps;

        // Calculate ranking score based on EPS, growth, and market cap
        let ranking_score = Self::calculate_ranking_score(current_eps, qoq_growth.unwrap_or(0.0), market_cap as f64, price_current);

        FrontendEPSData {
            id: Uuid::new_v4().to_string(),
            symbol,
            company_name,
            current_eps,
            qoq_growth: qoq_growth.unwrap_or(0.0), // Default to 0.0 if no QoQ growth calculated
            market_cap,
            price_current,
            volume,
            country,
            sector,
            ranking_score,
        }
    }

    /// Calculate ranking score based on multiple factors
    fn calculate_ranking_score(current_eps: f64, qoq_growth: f64, market_cap: f64, price: f64) -> f64 {
        // Weighted scoring algorithm
        let eps_weight = 0.3;
        let growth_weight = 0.4;
        let market_cap_weight = 0.2;
        let price_weight = 0.1;

        // Normalize values (simple approach)
        let eps_score = (current_eps * 10.0).min(100.0).max(0.0);
        let growth_score = (qoq_growth / 100.0 * 100.0).min(100.0).max(0.0);
        let market_cap_score = (market_cap / 1_000_000_000_000.0 * 100.0).min(100.0).max(0.0);
        let price_score = (price / 1000.0 * 100.0).min(100.0).max(0.0);

        (eps_score * eps_weight + growth_score * growth_weight + 
         market_cap_score * market_cap_weight + price_score * price_weight).round()
    }

    /// Debug logging for Taiwan stocks
    fn log_taiwan_stock_debug(symbol: &str, quarterly_eps: f64, ttm_eps: f64, basic_ttm: f64, stock: &TradingViewStock) {
        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            match data.get(idx) {
                Some(StockDataField::String(s)) => s.clone(),
                Some(StockDataField::Number(n)) => n.to_string(),
                Some(StockDataField::Integer(i)) => i.to_string(),
                _ => default.to_string(),
            }
        };

        let get_number = |data: &[StockDataField], idx: usize, default: f64| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => default,
            }
        };

        info!("📊 Taiwan Stock {} EPS Debug - Quarterly: {}, TTM: {}, Basic TTM: {}, Currency: {}", 
              symbol, quarterly_eps, ttm_eps, basic_ttm, 
              get_string(&stock.d, 11, "N/A")); // currency field
        
        // Log complete raw data array structure for debugging
        info!("🔍 RAW API RESPONSE DEBUG for {}: Full symbol: {}, Data array length: {}", 
              symbol, stock.s, stock.d.len());
        
        // Log all data fields with indices for field mapping verification
        for (idx, field) in stock.d.iter().enumerate() {
            let field_value = match field {
                StockDataField::String(s) => format!("String(\"{}\")", s),
                StockDataField::Number(n) => format!("Number({})", n),
                StockDataField::Integer(i) => format!("Integer({})", i),
                StockDataField::Boolean(b) => format!("Boolean({})", b),
                StockDataField::Array(_) => "Array([...])".to_string(),
                StockDataField::Object(_) => "Object({...})".to_string(),
                StockDataField::Null => "Null".to_string(),
            };
            info!("🔍 Index {}: {}", idx, field_value);
        }
        
        // Test different EPS field combinations and scaling factors
        info!("🔬 EPS FIELD COMBINATION TESTING for {}:", symbol);
        let test_fields = [
            (14, "quarterly_eps"), (18, "price_earnings_ttm"), (19, "ttm_eps"),
            (22, "forecast_fq"), (23, "forecast_next_fq")
        ];
        
        for (idx, field_name) in test_fields.iter() {
            let val = get_number(&stock.d, *idx, 0.0);
            if val > 0.0 {
                info!("🔬 Index {} ({}): {} | x10: {} | x25: {}", 
                      idx, field_name, val, val * 10.0, val * 25.0);
            }
        }
        
        // Currency and scale detection
        let fund_currency = get_string(&stock.d, 17, "N/A"); // fundamental_currency_code  
        let price = get_number(&stock.d, 6, 0.0); // close price
        info!("🔬 SCALE ANALYSIS - Price: {} TWD, Currency: {}, Fund_Currency: {}", 
              price, get_string(&stock.d, 11, "N/A"), fund_currency);
              
        // Calculate potential scaling factors based on expected vs actual
        let expected_range = (12.0, 15.0); // Expected TSMC EPS range
        if ttm_eps > 0.0 {
            let scale_factor_ttm = expected_range.0 / ttm_eps;
            info!("🔬 SCALING FACTOR - TTM EPS {} needs {}x to reach expected ~{}", 
                  ttm_eps, scale_factor_ttm, expected_range.0);
        }
        if quarterly_eps > 0.0 {
            let scale_factor_q = expected_range.0 / quarterly_eps;
            info!("🔬 SCALING FACTOR - Quarterly EPS {} needs {}x to reach expected ~{}", 
                  quarterly_eps, scale_factor_q, expected_range.0);
        }
    }

    /// Merge scanner data with WebSocket data, preferring WebSocket details
    pub fn merge_scanner_and_websocket_data(
        scanner_data: Vec<FrontendEPSData>,
        websocket_data: Vec<FrontendEPSData>
    ) -> Vec<FrontendEPSData> {
        use std::collections::HashMap;
        
        // Create a map of WebSocket data by symbol for fast lookup
        let websocket_map: HashMap<String, FrontendEPSData> = websocket_data
            .into_iter()
            .map(|item| (item.symbol.clone(), item))
            .collect();
        
        // Merge data, preferring WebSocket data when available
        scanner_data.into_iter().map(|mut scanner_item| {
            if let Some(websocket_item) = websocket_map.get(&scanner_item.symbol) {
                // Use WebSocket data for EPS details
                scanner_item.current_eps = websocket_item.current_eps;
                scanner_item.qoq_growth = websocket_item.qoq_growth;
                scanner_item.ranking_score = websocket_item.ranking_score;
                
                // Keep other data from scanner (company name, market cap, etc.)
                // as it may be more accurate from the scanner API
                debug!("Enhanced {} with WebSocket EPS data", scanner_item.symbol);
            }
            scanner_item
        }).collect()
    }

    /// Batch convert multiple stocks to EPS data
    pub fn batch_convert_to_eps_data(stocks: Vec<TradingViewStock>) -> Vec<StockDataResult<EPSGrowthData>> {
        stocks.into_iter()
            .map(Self::convert_to_eps_growth_data)
            .collect()
    }

    /// Batch convert multiple stocks to frontend format
    pub fn batch_convert_to_frontend_data(stocks: Vec<TradingViewStock>) -> Vec<FrontendEPSData> {
        stocks.into_iter()
            .map(Self::map_to_frontend_eps_data)
            .collect()
    }

    /// Validate and filter EPS data quality
    pub fn filter_quality_eps_data(eps_data_list: Vec<EPSGrowthData>) -> Vec<EPSGrowthData> {
        eps_data_list.into_iter()
            .filter(|eps_data| {
                // Apply quality filters
                if eps_data.has_quality_data() {
                    debug!("Including quality EPS data for: {}", eps_data.symbol);
                    true
                } else {
                    debug!("Filtering out {} due to incomplete data", eps_data.symbol);
                    false
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::entities::market_data::{TradingViewStock, StockDataField};

    fn create_test_stock() -> TradingViewStock {
        TradingViewStock {
            s: "NASDAQ:AAPL".to_string(),
            d: vec![
                StockDataField::String("AAPL".to_string()),        // name
                StockDataField::String("Apple Inc".to_string()),   // description
                StockDataField::String("logoid".to_string()),      // logoid
                StockDataField::String("streaming".to_string()),   // update_mode
                StockDataField::String("stock".to_string()),       // type
                StockDataField::Array(vec![]),                     // typespecs
                StockDataField::Number(150.0),                     // close
                StockDataField::Integer(100),                      // pricescale
                StockDataField::Integer(1),                        // minmov
                StockDataField::Boolean(false),                    // fractional
                StockDataField::Integer(0),                        // minmove2
                StockDataField::String("USD".to_string()),         // currency
                StockDataField::Number(2.5),                       // change
                StockDataField::Number(50000000.0),                // volume
                StockDataField::Number(3.25),                      // earnings_per_share_fq
                StockDataField::Number(1.2),                       // relative_volume_10d_calc
                StockDataField::Number(2500000000000.0),           // market_cap_basic
                StockDataField::String("USD".to_string()),         // fundamental_currency_code
                StockDataField::Number(23.5),                      // price_earnings_ttm
                StockDataField::Number(13.0),                      // earnings_per_share_diluted_ttm
                StockDataField::Number(12.5),                      // earnings_per_share_diluted_yoy_growth_ttm
                StockDataField::Number(1.8),                       // dividends_yield_current
                StockDataField::Number(3.50),                      // earnings_per_share_forecast_fq
                StockDataField::Number(3.75),                      // earnings_per_share_forecast_next_fq
                StockDataField::String("Technology".to_string()),  // sector.tr
                StockDataField::String("america".to_string()),     // market
                StockDataField::String("Technology".to_string()),  // sector
                StockDataField::String("Strong Buy".to_string()),  // AnalystRating
                StockDataField::String("Strong Buy".to_string()),  // AnalystRating.tr
                StockDataField::String("NASDAQ".to_string()),      // exchange
            ]
        }
    }

    #[test]
    fn test_eps_data_conversion() {
        let stock = create_test_stock();
        let result = TradingViewMapper::convert_to_eps_growth_data(stock);
        
        assert!(result.is_ok());
        let eps_data = result.unwrap();
        assert_eq!(eps_data.symbol, "AAPL");
        assert_eq!(eps_data.country, "america");
        assert!(eps_data.current_eps.is_some());
    }

    #[test]
    fn test_frontend_data_mapping() {
        let stock = create_test_stock();
        let frontend_data = TradingViewMapper::map_to_frontend_eps_data(stock);
        
        assert_eq!(frontend_data.symbol, "AAPL");
        assert_eq!(frontend_data.company_name, "Apple Inc");
        assert_eq!(frontend_data.current_eps, 3.25);
        assert_eq!(frontend_data.price_current, 150.0);
        assert!(frontend_data.ranking_score > 0.0);
    }

    #[test]
    fn test_eps_value_validation() {
        assert!(TradingViewMapper::is_valid_quarterly_eps_value(3.25));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(0.0));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(-1.0));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(f64::INFINITY));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(100000.0)); // Too high
    }

    #[test]
    fn test_ranking_score_calculation() {
        let score = TradingViewMapper::calculate_ranking_score(3.25, 12.5, 2500000000000.0, 150.0);
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_batch_conversion() {
        let stocks = vec![create_test_stock(), create_test_stock()];
        let frontend_data = TradingViewMapper::batch_convert_to_frontend_data(stocks);
        
        assert_eq!(frontend_data.len(), 2);
        assert!(frontend_data.iter().all(|data| data.symbol == "AAPL"));
    }

    #[test]
    fn test_merge_data() {
        let scanner_data = vec![
            FrontendEPSData {
                id: "1".to_string(),
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc".to_string(),
                current_eps: 3.0,
                qoq_growth: 10.0,
                market_cap: 2500000000000,
                price_current: 150.0,
                volume: 50000000,
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 80.0,
            }
        ];

        let websocket_data = vec![
            FrontendEPSData {
                id: "2".to_string(),
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc".to_string(),
                current_eps: 3.25, // Enhanced EPS from WebSocket
                qoq_growth: 12.5,  // Enhanced growth from WebSocket
                market_cap: 2500000000000,
                price_current: 150.0,
                volume: 50000000,
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 85.0, // Enhanced ranking from WebSocket
            }
        ];

        let merged = TradingViewMapper::merge_scanner_and_websocket_data(scanner_data, websocket_data);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].current_eps, 3.25); // Should use WebSocket value
        assert_eq!(merged[0].qoq_growth, 12.5);  // Should use WebSocket value
    }
}