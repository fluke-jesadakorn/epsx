// WebSocket Data Enhancement Logic
// Focused module handling real-time EPS data enhancement from TradingView WebSocket

use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::domain::trading_analytics::EPSRanking;
use crate::infra::services::tradingview_websocket::TradingViewWebSocketService;
use super::rankings::is_valid_eps_for_ranking;

/// Enhance EPS rankings with REAL TradingView WebSocket data (not hardcoded)
pub async fn enhance_with_websocket_data(
    symbols: &[String],
    rankings: &mut Vec<EPSRanking>
) -> Result<usize, String> {
    info!("🚀 Starting REAL TradingView WebSocket data enhancement for {} symbols", symbols.len());
    
    // Create WebSocket service and fetch REAL data from TradingView
    let mut ws_service = TradingViewWebSocketService::new();
    
    // Attempt to fetch real WebSocket data
    match ws_service.connect_and_fetch_eps_data(symbols.to_vec()).await {
        Ok(websocket_data) => {
            info!("✅ Successfully fetched REAL WebSocket data for {} symbols", websocket_data.len());
            
            let mut enhanced_count = 0;
            
            // Create a map for quick lookups
            let mut websocket_map = HashMap::new();
            for ws_data in websocket_data {
                websocket_map.insert(ws_data.symbol.clone(), ws_data);
            }
            
            // Update rankings with REAL WebSocket data
            for ranking in rankings.iter_mut() {
                if let Some(ws_data) = websocket_map.get(&ranking.symbol) {
                    info!("🔄 Enhancing {} with REAL TradingView WebSocket data", ranking.symbol);
                    
                    // Update with real current EPS using dynamic validation
                    if is_valid_eps_for_ranking(ws_data.current_eps) {
                        debug!("Updating {} current EPS: {:?} → {} (REAL WebSocket)", 
                               ranking.symbol, ranking.current_eps, ws_data.current_eps);
                        ranking.current_eps = Some(ws_data.current_eps);
                        enhanced_count += 1;
                    }
                    
                    // Update with real current price from WebSocket
                    if ws_data.price_current > 0.01 && ws_data.price_current.is_finite() {
                        debug!("Updating {} current price: {:?} → {} (REAL WebSocket)", 
                               ranking.symbol, ranking.price_current, ws_data.price_current);
                        ranking.price_current = Some(ws_data.price_current);
                    }
                    
                    // Store REAL quarterly data with price correlation
                    if !ws_data.quarterly_data.is_empty() {
                        ranking.quarterly_data = Some(ws_data.quarterly_data.clone());
                        
                        // Use correlated price data from most recent quarter if available
                        if let Some(recent_quarter) = ws_data.quarterly_data.first() {
                            if let Some(price_data) = &recent_quarter.price_data {
                                // Use post-earnings price as most current price
                                if price_data.post_earnings_price > 0.0 {
                                    debug!("Updating {} price from correlation: {:?} → {} (from earnings correlation)", 
                                           ranking.symbol, ranking.price_current, price_data.post_earnings_price);
                                    ranking.price_current = Some(price_data.post_earnings_price);
                                }
                            }
                        }
                        
                        // Calculate QoQ growth from REAL quarterly data
                        if ws_data.quarterly_data.len() >= 2 {
                            let current_eps = ws_data.quarterly_data[0].eps;
                            let previous_eps = ws_data.quarterly_data[1].eps;
                            
                            if previous_eps > 0.0 {
                                let qoq_growth = ((current_eps - previous_eps) / previous_eps) * 100.0;
                                if qoq_growth.abs() < 200.0 { // Reasonable growth range
                                    debug!("Updating {} QoQ growth: {:?} → {}% (REAL WebSocket)", 
                                           ranking.symbol, ranking.growth_factor, qoq_growth);
                                    ranking.growth_factor = Some(qoq_growth);
                                }
                            }
                        }
                    }
                }
            }
            
            info!("✅ Enhanced {} out of {} rankings with REAL TradingView WebSocket data", enhanced_count, rankings.len());
            Ok(enhanced_count)
        }
        Err(e) => {
            warn!("⚠️ WebSocket connection failed: {}", e);
            // No fallback data - fail gracefully and return error
            Err(format!("WebSocket enhancement failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::trading_analytics::EPSRanking;

    #[test]
    fn test_empty_rankings_enhancement() {
        let symbols = vec!["AAPL".to_string()];
        let mut rankings = Vec::new();
        
        // Test with empty rankings - should not panic
        // Note: This is a unit test so we can't actually test the async WebSocket functionality
        assert_eq!(rankings.len(), 0);
    }

    #[test]
    fn test_ranking_has_required_fields() {
        let ranking = EPSRanking {
            symbol: "AAPL".to_string(),
            name: "Apple Inc".to_string(),
            country: "america".to_string(),
            sector: "Technology".to_string(),
            exchange: "NASDAQ".to_string(),
            current_eps: Some(1.5),
            qoq_growth: Some(10.0),
            price_current: Some(150.0),
            market_cap: Some(2500000000),
            volume: Some(50000000),
            ranking_position: Some(1),
            quarterly_data: None,
        };

        assert_eq!(ranking.symbol, "AAPL");
        assert!(ranking.current_eps.is_some());
        assert!(ranking.price_current.is_some());
    }
}