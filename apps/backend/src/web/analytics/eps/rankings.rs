// Core EPS Rankings Business Logic
// Focused module handling EPS rankings endpoint and business logic

use axum::{
    extract::{Query, Extension},
    response::Json,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::core::errors::AppError;
use crate::dom::services::eps_ranking_service::{EPSRankingService, EPSRankingParams};
use super::{dto::*, enhancement::enhance_with_websocket_data};

/// GET /api/analytics/eps-rankings
/// Returns top EPS growth stocks with filtering and pagination
pub async fn get_eps_rankings(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<EPSRankingsApiResponse>, AppError> {
    debug!("EPS Rankings API called with params: {:?}", params);
    
    // Convert query params to service params with defaults
    let service_params = EPSRankingParams {
        country: params.country.clone(),
        sector: params.sector.clone(),
        sort_by: params.sort_by.clone().or(Some("qoq_growth".to_string())),
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(50),
        min_eps: params.min_eps,
        min_growth: params.min_growth,
    };

    debug!("Converted to service params: {:?}", service_params);

    // Validate parameters
    service.validate_ranking_params(&service_params)?;

    // Log request details for debugging
    info!("Processing EPS rankings request - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          service_params.country, service_params.sort_by, service_params.page, service_params.limit);

    // Get rankings from service with enhanced WebSocket data when available
    let start_time = std::time::Instant::now();
    let mut result = service.get_eps_rankings(service_params).await?;
    let duration = start_time.elapsed();
    
    // For small requests (≤20), enhance with WebSocket data for accuracy
    if result.rankings.len() <= 20 && result.rankings.len() > 0 {
        debug!("Enhancing {} rankings with WebSocket EPS data", result.rankings.len());
        
        // Extract symbols for WebSocket enhancement
        let symbols: Vec<String> = result.rankings.iter()
            .map(|r| r.symbol.clone())
            .collect();
        
        // Try to enhance with WebSocket data
        match enhance_with_websocket_data(&symbols, &mut result.rankings).await {
            Ok(enhanced_count) => {
                info!("Enhanced {} rankings with WebSocket data", enhanced_count);
            }
            Err(e) => {
                warn!("Failed to enhance with WebSocket data: {}, using screener data", e);
            }
        }
    }

    // Log performance metrics
    debug!("EPS rankings query completed in {:?}", duration);
    info!("Returning {} EPS rankings for page {} (total: {})", 
          result.rankings.len(), result.pagination.page, result.pagination.total);

    // Convert to API response format
    let api_response = EPSRankingsApiResponse::from(result);

    Ok(Json(api_response))
}

/// Convert StockScreeningResult to legacy EPSRanking format
/// This function bridges legacy market data entities to the expected EPSRanking structure
pub fn convert_screening_result_to_eps_ranking(
    result: crate::dom::entities::market_data::StockScreeningResult
) -> crate::dom::entities::eps_growth::EPSRanking {
    use crate::dom::entities::eps_growth::EPSRanking;
    
    // Parse numeric values with fallback for string data
    let current_eps = parse_f64_from_string(&result.current_metric);
    let growth_factor = parse_f64_from_string(&result.growth_factor);
    let market_cap = result.market_size.parse::<i64>().ok();
    let volume = result.activity_score.parse::<i64>().ok();
    let ranking_position = result.value_index.parse::<i32>().ok();
    
    // Calculate price from available metrics (if available)
    let price_current = parse_f64_from_string(&result.predicted_metric);
    
    EPSRanking {
        symbol: result.symbol,
        name: result.name,
        country: result.country,
        sector: result.sector,
        exchange: result.exchange,
        current_eps,
        growth_factor,
        price_current,
        market_cap,
        volume,
        ranking_position,
        quarterly_data: None, // Will be populated by WebSocket enhancement if needed
        next_earnings_date: None, // Will be populated from external data
        last_earnings_date: result.last_earnings_date,
    }
}

/// Helper function to parse f64 from string fields with fallback handling
fn parse_f64_from_string(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().filter(|f| f.is_finite() && !f.is_nan())
}

/// Dynamic EPS validation for ranking updates - no hardcoded country/stock limits
pub fn is_valid_eps_for_ranking(eps: f64) -> bool {
    // Basic sanity checks
    if !eps.is_finite() || eps <= 0.0 {
        return false;
    }

    // Allow very wide range to handle all markets and currencies
    // US stocks: 0.01 to 50+ USD per share  
    // International stocks: much higher (Taiwan stocks in TWD, Japanese stocks in JPY)
    // Accept any reasonable positive value up to 50,000 to handle all currencies and markets
    if eps > 50000.0 {
        warn!("EPS value {} is extremely high, might be an error", eps);
        return false;
    }

    // Accept small values too (penny stocks, recent IPOs, etc.)
    if eps < 0.001 {
        warn!("EPS value {} is very small, might be noise", eps);
        return false;
    }

    // All values in reasonable range are valid - no country/stock specific limits
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eps_ranking_query_params() {
        // Test default values
        let params = EPSRankingQueryParams {
            page: None,
            limit: None,
            country: None,
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        let service_params = EPSRankingParams {
            country: params.country,
            sector: params.sector,
            sort_by: params.sort_by.or(Some("qoq_growth".to_string())),
            page: params.page.unwrap_or(1),
            limit: params.limit.unwrap_or(50),
            min_eps: params.min_eps,
            min_growth: params.min_growth,
        };

        assert_eq!(service_params.page, 1);
        assert_eq!(service_params.limit, 50);
        assert_eq!(service_params.sort_by, Some("qoq_growth".to_string()));
    }

    #[test]
    fn test_eps_validation() {
        assert!(is_valid_eps_for_ranking(1.0));
        assert!(is_valid_eps_for_ranking(0.1));
        assert!(is_valid_eps_for_ranking(100.0));
        assert!(!is_valid_eps_for_ranking(0.0));
        assert!(!is_valid_eps_for_ranking(-1.0));
        assert!(!is_valid_eps_for_ranking(f64::NAN));
        assert!(!is_valid_eps_for_ranking(f64::INFINITY));
    }
}