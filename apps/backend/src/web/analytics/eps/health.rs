// Health Checks and Debug Endpoints
// Focused module handling EPS service health monitoring and debugging

use axum::{
    extract::Extension,
    response::Json,
};
use std::sync::Arc;
use tracing::{debug, info, error};

use crate::core::errors::{AppError, ErrorKind};
use crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService;
use crate::infrastructure::adapters::services::tradingview_websocket::TradingViewWebSocketService;
use super::types::EPSHealthResponse;

/// GET /api/analytics/eps-rankings/health
/// Health check endpoint for EPS analytics service
pub async fn eps_health_check(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<EPSHealthResponse>, AppError> {
    debug!("EPS service health check requested");

    // Try to get available countries as a health indicator
    match service.get_available_countries().await {
        Ok(countries) => {
            let response = EPSHealthResponse {
                status: "healthy".to_string(),
                message: "EPS analytics service is operational".to_string(),
                available_countries: countries.len(),
            };
            info!("EPS service health check passed - {} countries available", countries.len());
            Ok(Json(response))
        }
        Err(e) => {
            error!("EPS service health check failed: {:?}", e);
            let response = EPSHealthResponse {
                status: "unhealthy".to_string(),
                message: format!("EPS analytics service error: {}", e),
                available_countries: 0,
            };
            Ok(Json(response))
        }
    }
}

/// POST /api/analytics/eps-rankings/debug-eps-raw
/// Debug raw quarterly EPS values (no correction applied)
pub async fn debug_eps_correction() -> Result<Json<serde_json::Value>, AppError> {
    info!("Raw EPS debug test triggered");
    
    let test_cases = vec![
        ("2330", "taiwan", 0.526),   // TSMC quarterly
        ("LLY", "america", 6.31),    // LLY quarterly
        ("NVDA", "america", 2.5),    // NVDA quarterly
        ("AAPL", "america", 1.5),    // AAPL quarterly
    ];
    
    let mut results = Vec::new();
    
    for (symbol, country, raw_eps) in test_cases {
        results.push(serde_json::json!({
            "symbol": symbol,
            "country": country,
            "quarterly_eps": raw_eps,
            "note": "Using raw quarterly EPS directly - no TTM fallback or correction"
        }));
        
        info!("Raw quarterly EPS: {} ({}) = {}", symbol, country, raw_eps);
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Raw EPS debug test completed - simplified system using quarterly EPS only",
        "test_cases": results
    })))
}

/// POST /api/analytics/eps-rankings/debug-ranking-data
/// Debug actual ranking data structure for specific symbols
pub async fn debug_ranking_data(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<serde_json::Value>, AppError> {
    info!("Ranking data debug test triggered");
    
    // Get actual ranking data for TSMC and LLY
    match service.get_eps_rankings(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingParams {
        sector: None,
        country: Some("taiwan".to_string()),
        min_eps: None,
        min_growth: None,
        page: 1,
        limit: 5,
        sort_by: None,
    }).await {
        Ok(rankings_response) => {
            let mut results = Vec::new();
            
            for ranking in &rankings_response.rankings {
                if ranking.symbol == "2330" || ranking.symbol == "LLY" {
                    results.push(serde_json::json!({
                        "symbol": ranking.symbol,
                        "company_name": ranking.name,
                        "current_eps": ranking.current_eps,
                        "previous_eps": None::<f64> /* eps_previous not available in DDD structure */,
                        "growth_rate": ranking.growth_factor
                    }));
                    
                    info!("🔍 Ranking debug - Symbol: {}, Company: '{}', Current EPS: {:.3}, Growth Rate: {:.2}%", 
                          ranking.symbol, ranking.name, 
                          ranking.current_eps.unwrap_or(0.0),
                          ranking.growth_factor.unwrap_or(0.0));
                }
            }
            
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Ranking data debug completed",
                "rankings_found": results
            })))
        },
        Err(e) => {
            error!("Failed to get ranking data: {:?}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "message": format!("Failed to get ranking data: {}", e),
                "rankings_found": []
            })))
        }
    }
}

/// POST /api/analytics/eps-rankings/websocket-test
/// Test WebSocket EPS data extraction
pub async fn debug_websocket_eps() -> Result<Json<serde_json::Value>, AppError> {
    info!("WebSocket EPS test triggered");
    
    // Create WebSocket service
    let mut ws_service = TradingViewWebSocketService::new();
    
    info!("Starting WebSocket connection for NVDA EPS data...");
    
    // Connect and fetch EPS data for NVDA
    let symbols = vec!["AAPL".to_string()]; // Use any symbol for testing
    match ws_service.connect_and_fetch_eps_data(symbols).await {
        Ok(eps_data) => {
            info!("WebSocket data collection completed - {} entries", eps_data.len());
            
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "WebSocket EPS test completed successfully",
                "data": {
                    "eps_entries_collected": eps_data.len(),
                    "eps_data_sample": eps_data.into_iter().take(3).map(|eps| {
                        serde_json::json!({
                            "symbol": eps.symbol,
                            "current_eps": eps.current_eps,
                            "quarterly_eps": eps.quarterly_eps,
                            "price_current": eps.price_current,
                            "volume": eps.volume
                        })
                    }).collect::<Vec<_>>()
                }
            })))
        }
        Err(e) => {
            error!("WebSocket EPS test failed: {:?}", e);
            Err(AppError::new(ErrorKind::ExternalServiceError, format!("WebSocket test failed: {}", e)))
        }
    }
}

/// POST /api/analytics/eps-rankings/sync
/// Manually trigger EPS data synchronization from TradingView
pub async fn trigger_eps_sync() -> Result<Json<serde_json::Value>, AppError> {
    info!("Manual EPS sync triggered");
    
    // Create TradingView service and processor  
    use crate::config::Config;
    use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
    
    let config = match Config::from_env() {
        Ok(config) => std::sync::Arc::new(config),
        Err(e) => {
            tracing::warn!("Failed to load config, using fallback: {:?}", e);
            std::sync::Arc::new(get_default_config())
        }
    };
    let _tradingview_service = std::sync::Arc::new(TradingViewApiService::new(config));
    
    // Note: InfraFactory creation would be handled in actual implementation
    // let _infra_factory = InfraFactory { db_pool, cache, firebase_admin };
    
    info!("Starting manual EPS data processing...");
    // TODO: Implement EPSDataProcessor module
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "EPS processor not yet implemented"
    })))
}

/// Get default configuration for fallback
fn get_default_config() -> crate::config::Config {
    crate::config::get_fallback_config()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = get_default_config();
        assert_eq!(config.backend_url, "http://localhost:8080");
        assert_eq!(config.database_url, "postgresql://localhost/epsx");
    }
}