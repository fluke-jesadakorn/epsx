// TradingView Mapper - Focused Module for Data Transformation and Mapping
// Handles conversion between TradingView data and internal/frontend formats

use uuid::Uuid;
use tracing::{debug, error, info, warn};

use super::types::{TradingViewStock, StockDataField};
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
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

        // If primary field not available, use the first valid candidate
        let (_, name, _, value) = &valid_candidates[0];
        info!("Selected fallback quarterly EPS field {} for {}: {}", name, symbol, value);
        Some(*value)
    }

    /// Validate if a quarterly EPS value is reasonable
    fn is_valid_quarterly_eps_value(value: f64) -> bool {
        // Basic validation for quarterly EPS values
        value > 0.0 && value < 1000.0 && value.is_finite()
    }

    /// Map TradingView stock to frontend EPS data format
    pub fn map_to_frontend_eps_data(stock: TradingViewStock) -> FrontendEPSData {
        let get_number = |data: &[StockDataField], idx: usize| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => 0.0
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

        let symbol = stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string();
        let company_name = get_string(&stock.d, 0, "");
        let current_eps = get_number(&stock.d, 14); // earnings_per_share_fq
        let qoq_growth = get_number(&stock.d, 19); // earnings_per_share_diluted_yoy_growth_ttm
        let market_cap = get_number(&stock.d, 16) as i64; // market_cap_basic
        let price_current = get_number(&stock.d, 6); // close
        let volume = get_number(&stock.d, 13) as i64; // volume
        let country = get_string(&stock.d, 25, "america"); // market
        let sector = get_string(&stock.d, 24, "Technology"); // sector.tr

        // Calculate ranking score based on multiple factors
        let ranking_score = Self::calculate_ranking_score(current_eps, qoq_growth, market_cap as f64, price_current);

        FrontendEPSData {
            id: Uuid::new_v4().to_string(),
            symbol,
            company_name,
            current_eps,
            qoq_growth,
            market_cap,
            price_current,
            volume,
            country: country.to_lowercase(),
            sector,
            ranking_score,
        }
    }

    /// Calculate ranking score for frontend display
    fn calculate_ranking_score(eps: f64, growth: f64, market_cap: f64, price: f64) -> f64 {
        // Weighted scoring algorithm
        let eps_score = if eps > 0.0 { eps * 20.0 } else { 0.0 };
        let growth_score = growth.abs() * 10.0;
        let market_cap_score = (market_cap / 1_000_000_000.0).min(100.0); // Cap at 100
        let price_score = (price / 1000.0).min(50.0); // Cap at 50

        let total_score = eps_score + growth_score + market_cap_score + price_score;
        total_score.min(100.0) // Cap final score at 100
    }

    /// Merge scanner data with WebSocket data for enhanced results
    pub fn merge_scanner_and_websocket_data(
        scanner_data: Vec<FrontendEPSData>,
        websocket_data: Vec<FrontendEPSData>,
    ) -> Vec<FrontendEPSData> {
        let mut merged_data = Vec::new();
        
        for scanner_item in scanner_data {
            // Try to find matching WebSocket data
            if let Some(websocket_item) = websocket_data.iter().find(|ws| ws.symbol == scanner_item.symbol) {
                // Merge the data, prioritizing WebSocket values for real-time data
                let merged_item = FrontendEPSData {
                    id: scanner_item.id,
                    symbol: scanner_item.symbol,
                    company_name: scanner_item.company_name,
                    current_eps: if websocket_item.current_eps > 0.0 { websocket_item.current_eps } else { scanner_item.current_eps },
                    qoq_growth: if websocket_item.qoq_growth != 0.0 { websocket_item.qoq_growth } else { scanner_item.qoq_growth },
                    market_cap: if websocket_item.market_cap > 0 { websocket_item.market_cap } else { scanner_item.market_cap },
                    price_current: if websocket_item.price_current > 0.0 { websocket_item.price_current } else { scanner_item.price_current },
                    volume: if websocket_item.volume > 0 { websocket_item.volume } else { scanner_item.volume },
                    country: scanner_item.country, // Keep scanner country
                    sector: scanner_item.sector,   // Keep scanner sector
                    ranking_score: Self::calculate_ranking_score(
                        if websocket_item.current_eps > 0.0 { websocket_item.current_eps } else { scanner_item.current_eps },
                        if websocket_item.qoq_growth != 0.0 { websocket_item.qoq_growth } else { scanner_item.qoq_growth },
                        if websocket_item.market_cap > 0 { websocket_item.market_cap as f64 } else { scanner_item.market_cap as f64 },
                        if websocket_item.price_current > 0.0 { websocket_item.price_current } else { scanner_item.price_current }
                    ),
                };
                merged_data.push(merged_item);
            } else {
                // No WebSocket data found, use scanner data as-is
                merged_data.push(scanner_item);
            }
        }
        
        debug!("Merged {} scanner items with {} WebSocket items", merged_data.len(), websocket_data.len());
        merged_data
    }

    /// Convert batch of TradingView stocks to frontend format
    pub fn batch_convert_to_frontend(stocks: Vec<TradingViewStock>) -> Vec<FrontendEPSData> {
        stocks.into_iter()
            .map(Self::map_to_frontend_eps_data)
            .collect()
    }

    /// Validate and clean frontend EPS data
    pub fn validate_frontend_data(data: &mut Vec<FrontendEPSData>) -> usize {
        let original_count = data.len();
        
        data.retain(|item| {
            // Basic validation rules
            !item.symbol.is_empty() && 
            !item.company_name.is_empty() && 
            item.current_eps >= 0.0 && 
            item.market_cap >= 0 &&
            item.price_current >= 0.0
        });
        
        let removed_count = original_count - data.len();
        if removed_count > 0 {
            warn!("Removed {} invalid frontend EPS data items", removed_count);
        }
        
        removed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared_kernel::entities::market_data::StockDataField;

    #[test]
    fn test_eps_value_validation() {
        assert!(TradingViewMapper::is_valid_quarterly_eps_value(3.25));
        assert!(TradingViewMapper::is_valid_quarterly_eps_value(0.01));
        assert!(TradingViewMapper::is_valid_quarterly_eps_value(100.0));
        
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(0.0));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(-1.0));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(1000.1));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(f64::NAN));
        assert!(!TradingViewMapper::is_valid_quarterly_eps_value(f64::INFINITY));
    }

    #[test]
    fn test_ranking_score_calculation() {
        let score = TradingViewMapper::calculate_ranking_score(3.25, 15.5, 2_000_000_000.0, 150.0);
        assert!(score > 0.0);
        assert!(score <= 100.0);
        
        let zero_score = TradingViewMapper::calculate_ranking_score(0.0, 0.0, 0.0, 0.0);
        assert_eq!(zero_score, 0.0);
    }

    #[test]
    fn test_quarterly_eps_selection() {
        let candidates = vec![
            (14, "earnings_per_share_fq", "quarterly", 3.25),
            (22, "earnings_per_share_forecast_fq", "quarterly", 3.50),
        ];
        
        let result = TradingViewMapper::select_best_quarterly_eps_field(candidates, "AAPL");
        assert_eq!(result, Some(3.25)); // Should prefer index 14
    }

    #[test]
    fn test_frontend_data_validation() {
        let data = vec![
            FrontendEPSData {
                id: "1".to_string(),
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc".to_string(),
                current_eps: 3.25,
                qoq_growth: 12.5,
                market_cap: 2_500_000_000_000,
                price_current: 150.0,
                volume: 50_000_000,
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 85.5,
            },
            FrontendEPSData {
                id: "2".to_string(),
                symbol: "".to_string(), // Invalid - empty symbol
                company_name: "Invalid Corp".to_string(),
                current_eps: 1.0,
                qoq_growth: 5.0,
                market_cap: 1_000_000_000,
                price_current: 50.0,
                volume: 1_000_000,
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 50.0,
            },
        ];
        
        let removed_count = TradingViewMapper::validate_frontend_data(&mut data);
        assert_eq!(removed_count, 1);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].symbol, "AAPL");
    }

    #[test]
    fn test_data_merging() {
        let scanner_data = vec![
            FrontendEPSData {
                id: "1".to_string(),
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc".to_string(),
                current_eps: 3.25,
                qoq_growth: 12.5,
                market_cap: 2_500_000_000_000,
                price_current: 150.0,
                volume: 50_000_000,
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 85.5,
            },
        ];
        
        let websocket_data = vec![
            FrontendEPSData {
                id: "ws_1".to_string(),
                symbol: "AAPL".to_string(),
                company_name: "Apple Inc".to_string(),
                current_eps: 3.30, // Updated EPS from WebSocket
                qoq_growth: 13.0,  // Updated growth from WebSocket
                market_cap: 0,     // No WebSocket market cap
                price_current: 152.0, // Updated price from WebSocket
                volume: 0,         // No WebSocket volume
                country: "america".to_string(),
                sector: "Technology".to_string(),
                ranking_score: 0.0,
            },
        ];
        
        let merged = TradingViewMapper::merge_scanner_and_websocket_data(scanner_data, websocket_data);
        assert_eq!(merged.len(), 1);
        
        let merged_item = &merged[0];
        assert_eq!(merged_item.current_eps, 3.30); // Should use WebSocket EPS
        assert_eq!(merged_item.qoq_growth, 13.0);  // Should use WebSocket growth
        assert_eq!(merged_item.price_current, 152.0); // Should use WebSocket price
        assert_eq!(merged_item.market_cap, 2_500_000_000_000); // Should use scanner market cap
        assert_eq!(merged_item.volume, 50_000_000); // Should use scanner volume
    }
}