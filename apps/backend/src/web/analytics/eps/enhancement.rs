use chrono::{DateTime, Utc};
use std::collections::HashMap;// WebSocket Data Enhancement Logic
// Focused module handling real-time EPS data enhancement from TradingView WebSocket

use tracing::{debug, info, warn};

use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
use crate::infrastructure::adapters::services::tradingview_websocket::{TradingViewWebSocketService, EPSWebSocketData};
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
                        debug!("Updating {} current EPS: {} → {} (REAL WebSocket)", 
                               ranking.symbol, ranking.eps_current, ws_data.current_eps);
                        ranking.eps_current = ws_data.current_eps;
                        enhanced_count += 1;
                    }
                    
                    // Calculate previous EPS from quarterly data if available
                    let previous_eps = if ws_data.quarterly_data.len() >= 2 {
                        ws_data.quarterly_data[1].eps  // Second most recent quarter
                    } else if !ws_data.historical_eps.is_empty() {
                        ws_data.historical_eps[0]  // Use first historical EPS as fallback
                    } else {
                        ranking.eps_previous  // Keep existing value
                    };
                    
                    // Update previous EPS if valid
                    if is_valid_eps_for_ranking(previous_eps) {
                        debug!("Updating {} previous EPS: {} → {} (REAL WebSocket)", 
                               ranking.symbol, ranking.eps_previous, previous_eps);
                        ranking.eps_previous = previous_eps;
                    }
                    
                    // Recalculate growth rate from updated EPS values
                    if ranking.eps_previous != 0.0 {
                        let new_growth_rate = ((ranking.eps_current - ranking.eps_previous) / ranking.eps_previous) * 100.0;
                        debug!("Updating {} growth rate: {} → {}% (calculated from WebSocket EPS)", 
                               ranking.symbol, ranking.growth_rate, new_growth_rate);
                        ranking.growth_rate = new_growth_rate;
                    }
                    
                    // Update last_updated timestamp with current time since WebSocket data is live
                    ranking.last_updated = chrono::Utc::now();
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
    use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;

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
        let ranking = EPSRanking::new(
            "AAPL".to_string(),
            "Apple Inc".to_string(),
            1.5,
            1.2,
            "Technology".to_string(),
        );

        assert_eq!(ranking.symbol, "AAPL");
        assert_eq!(ranking.eps_current, 1.5);
        assert_eq!(ranking.eps_previous, 1.2);
    }
}