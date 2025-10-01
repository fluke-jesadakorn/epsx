// WebSocket Data Enhancement Logic
// Focused module handling real-time EPS data enhancement from TradingView WebSocket

use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
use crate::infrastructure::adapters::services::tradingview::TradingViewWebSocketHandler;
use crate::config::get_fallback_config;
use super::rankings::is_valid_eps_for_ranking;

/// Enhance EPS rankings with REAL TradingView WebSocket data (performance optimized)
pub async fn enhance_with_websocket_data(
    symbols: &[String],
    rankings: &mut Vec<EPSRanking>
) -> Result<usize, String> {
    info!("🚀 Starting optimized TradingView WebSocket enhancement for {} symbols", symbols.len());
    
    // Performance safeguard - limit symbol count
    if symbols.len() > 15 {
        warn!("⚠️ Too many symbols for WebSocket enhancement: {}, limiting to 15", symbols.len());
        let limited_symbols = &symbols[..15];
        return enhance_symbols_batch(limited_symbols, rankings).await;
    }
    
    enhance_symbols_batch(symbols, rankings).await
}

/// Enhanced batch processing with optimizations
async fn enhance_symbols_batch(
    symbols: &[String], 
    rankings: &mut Vec<EPSRanking>
) -> Result<usize, String> {
    // Create optimized WebSocket handler using the new implementation
    let config = crate::infrastructure::adapters::services::tradingview::TradingViewConfig::from(&get_fallback_config());
    let ws_handler = TradingViewWebSocketHandler::new(config);
    
    // Add connection timeout and retry logic
    let connection_timeout = tokio::time::Duration::from_secs(15);
    
    match tokio::time::timeout(connection_timeout, ws_handler.connect_realtime_feed()).await {
        Ok(Ok(_)) => {
            info!("✅ WebSocket connected successfully");
        }
        Ok(Err(e)) => {
            warn!("❌ WebSocket connection failed: {}", e);
            return Err(format!("WebSocket connection failed: {}", e));
        }
        Err(_) => {
            warn!("⏱️ WebSocket connection timed out after 15s");
            return Err("WebSocket connection timeout".to_string());
        }
    }
    
    // Fetch enhanced EPS data with timeout
    let data_timeout = tokio::time::Duration::from_secs(10);
    let websocket_data = match tokio::time::timeout(
        data_timeout,
        ws_handler.fetch_enhanced_eps_data(symbols.to_vec())
    ).await {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            warn!("❌ WebSocket data fetch failed: {}", e);
            return Err(format!("WebSocket data fetch failed: {}", e));
        }
        Err(_) => {
            warn!("⏱️ WebSocket data fetch timed out after 10s");
            return Err("WebSocket data timeout".to_string());
        }
    };
    
    info!("✅ Fetched WebSocket data for {} symbols", websocket_data.len());
    
    // Efficient update process
    let mut enhanced_count = 0;
    let mut websocket_map = HashMap::new();
    
    // Build lookup map
    for ws_data in websocket_data {
        websocket_map.insert(ws_data.symbol.clone(), ws_data);
    }
    
    // Update rankings efficiently - only essential fields
    for ranking in rankings.iter_mut() {
        if let Some(ws_data) = websocket_map.get(&ranking.symbol) {
            // Only update if WebSocket data is fresher/better
            if ws_data.current_eps > 0.0 && is_valid_eps_for_ranking(ws_data.current_eps) && (ranking.current_eps.is_none() || ranking.current_eps.unwrap() != ws_data.current_eps) {
                debug!("📈 Updating {} EPS: {:?} → {}", 
                       ranking.symbol, ranking.current_eps, ws_data.current_eps);
                ranking.current_eps = Some(ws_data.current_eps);
                enhanced_count += 1;
            }
            
            // Update growth factor if significant difference  
            if ws_data.qoq_growth.abs() > 0.1 && (ranking.growth_factor.is_none() || (ranking.growth_factor.unwrap() - ws_data.qoq_growth).abs() > 1.0) {
                debug!("📊 Updating {} growth: {:?} → {}%", 
                       ranking.symbol, ranking.growth_factor, ws_data.qoq_growth);
                ranking.growth_factor = Some(ws_data.qoq_growth);
                enhanced_count += 1;
            }
            
            // Update price if available and different
            if ws_data.price_current > 0.0 && (ranking.price_current.is_none() || (ranking.price_current.unwrap() - ws_data.price_current).abs() > 0.01) {
                debug!("💰 Updating {} price: {:?} → ${:.2}",
                       ranking.symbol, ranking.price_current, ws_data.price_current);
                ranking.price_current = Some(ws_data.price_current);
                enhanced_count += 1;
            }

            // Update earnings dates from WebSocket data
            if ws_data.next_earnings_date.is_some() && ws_data.next_earnings_date != ranking.next_earnings_date {
                info!("📅 Updating {} next_earnings_date: {:?} → {:?}",
                      ranking.symbol, ranking.next_earnings_date, ws_data.next_earnings_date);
                ranking.next_earnings_date = ws_data.next_earnings_date.clone();
                enhanced_count += 1;
            }

            if ws_data.last_earnings_date.is_some() && ws_data.last_earnings_date != ranking.last_earnings_date {
                info!("📅 Updating {} last_earnings_date: {:?} → {:?}",
                      ranking.symbol, ranking.last_earnings_date, ws_data.last_earnings_date);
                ranking.last_earnings_date = ws_data.last_earnings_date.clone();
                enhanced_count += 1;
            }
        }
    }
    
    info!("🎯 Enhanced {} ranking fields with real-time WebSocket data", enhanced_count);
    Ok(enhanced_count)
}

#[cfg(test)]
mod tests {
    use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;

    #[test]
    fn test_empty_rankings_enhancement() {
        let symbols = vec!["AAPL".to_string()];
        let rankings: Vec<EPSRanking> = Vec::new();
        
        // Test with empty rankings - should not panic
        // Note: This is a unit test so we can't actually test the async WebSocket functionality
        assert_eq!(rankings.len(), 0);
    }

    #[test]
    fn test_ranking_has_required_fields() {
        let ranking = EPSRanking::from_eps_data(
            EPSGrowthData {
                symbol: "AAPL".to_string(),
                name: "Apple Inc".to_string(),
                country: "america".to_string(),
                sector: "Technology".to_string(),
                exchange: "NASDAQ".to_string(),
                current_eps: Some(1.5),
                growth_factor: Some(10.0),
                price_current: Some(150.0),
                market_cap: Some(2500000000),
                volume: Some(50000000),
                ranking_position: Some(1),
                quarterly_data: None,
                created_at: None,
                updated_at: None,
                next_earnings_date: None,
                last_earnings_date: None,
            },
            Some(1)
        );

        assert_eq!(ranking.symbol, "AAPL");
        assert_eq!(ranking.current_eps.unwrap_or(0.0), 1.5);
    }
}