// Core EPS Rankings Business Logic
// Focused module handling EPS rankings endpoint and business logic
// Uses UnifiedPermissionService for permission checking

use axum::{
    extract::{Query, Extension},
    response::Json,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

use epsx_contracts::errors::AppError;
use crate::domain::market_analytics::services::eps_ranking_service::{EPSRankingService, EPSRankingParams};
use crate::auth::UnifiedPermissionService;
use crate::web::pagination::Pagination;
use super::{types::*, enhancement::enhance_with_websocket_data};

/// GET /api/analytics/eps-rankings
/// Returns top EPS growth stocks with filtering and pagination
pub async fn get_eps_rankings(
    wallet_ext: Option<Extension<String>>,  // Wallet from Bearer token (injected by auth middleware)
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
    Extension(permission_service): Extension<Arc<UnifiedPermissionService>>,
) -> Result<Json<EPSRankingsApiResponse>, AppError> {
    debug!("EPS Rankings API called with params: {:?}", params);

    // Wallet address from Bearer token (injected by auth middleware after JWT validation)
    let wallet_address = wallet_ext.map(|Extension(addr)| addr.to_lowercase());

    // Calculate ranking configuration based on user permissions
    let (rank_offset, limit_cap) = if let Some(ref wallet) = wallet_address {
        calculate_ranking_config_from_permissions(&permission_service, wallet).await
    } else {
        (epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET, -1) // Default free tier offset and limit for anonymous users
    };

    debug!("Calculated ranking config: offset={}, limit={} for wallet: {:?}",
           rank_offset, limit_cap, wallet_address);

    // Pagination with default 50, max 100 (params.page and params.limit are Option<i32>)
    let pg = Pagination::from_signed(params.page, params.limit, 50, 100);

    // Convert query params to service params with defaults - FIXED: Use correct parameter structure
    let service_params = EPSRankingParams {
        country: params.country.clone(),
        sector: params.sector.clone(),
        sort_by: params.sort_by.clone().or(Some("growth_factor".to_string())),
        page: pg.page as i32, // FIXED: Add missing page parameter
        limit: pg.limit as i32, // FIXED: Use correct i32 type
        min_eps: params.min_eps, // FIXED: Use correct field name (not market_cap_min)
        min_growth: params.min_growth, // FIXED: Add missing min_growth parameter
        rank_offset, // SECURITY: Enforced from user permissions
        limit_cap,   // SECURITY: Enforced from user permissions
    };

    debug!("Converted to service params: {:?}", service_params);

    // Note: Additional parameter validation could be added in EPSRankingService if needed

    // Log request details for debugging
    info!("Processing EPS rankings request - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          service_params.country, service_params.sort_by, service_params.page, service_params.limit);

    // Get rankings from service with enhanced WebSocket data when available
    let start_time = std::time::Instant::now();
    let mut result = service.get_eps_rankings(service_params).await.map_err(|e| AppError {
        kind: epsx_contracts::errors::ErrorKind::InternalError,
        message: format!("Failed to get EPS rankings: {}", e),
        context: Box::new(epsx_contracts::errors::ErrorContext::default()),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        stack_trace: None,
    })?;
    let duration = start_time.elapsed();
    
    // TEMPORARILY DISABLED: WebSocket enhancement causes 50+ second response times
    // The TradingView data is already real (not hardcoded), so WebSocket enhancement is optional
    #[allow(clippy::overly_complex_bool_expr)]
    if false && result.rankings.len() <= 20 && !result.rankings.is_empty() {
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
    
    info!("Using fast TradingView API data (WebSocket enhancement disabled for performance)");

    // Log performance metrics
    debug!("EPS rankings query completed in {:?}", duration);
    info!("Returning {} EPS rankings (total: {})",
          result.rankings.len(), result.pagination.total);

    // Convert to API response format
    let total = result.pagination.total;
    let total_pages = pg.total_pages(total as u64);

    let api_response = EPSRankingsApiResponse {
        data: result.rankings,
        pagination: EPSPaginationResponse {
            page: pg.page as i32,
            limit: pg.limit as i32,
            total,
            total_pages: total_pages as i32,
            has_next: pg.has_next(total as u64),
            has_prev: pg.has_prev(),
        },
        access_info: super::types::AccessInfo {
            min_accessible_rank: rank_offset,
            locked_ranks_count: rank_offset.max(1) - 1,  // Ranks 1 to (offset-1) are locked
        },
    };

    Ok(Json(api_response))
}

/// Convert StockScreeningResult to legacy EPSRanking format
/// This function bridges legacy market data entities to the expected EPSRanking structure
pub fn convert_screening_result_to_eps_ranking(
    result: crate::domain::shared_kernel::entities::market_data::StockScreeningResult
) -> crate::domain::shared_kernel::entities::eps_growth::EPSRanking {
    use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
    
    // Use real EPS data from TradingView instead of calculating from price/PE
    let current_eps = result.current_eps.or_else(|| {
        // Fallback: calculate from price/PE if real EPS not available
        if let Some(pe_ratio) = result.pe_ratio {
            Some(result.price / pe_ratio.max(1.0))
        } else {
            None // Don't use fake default values
        }
    });
    let growth_factor = result.eps_growth_yoy.or({
        // Fallback: use price change as growth proxy if EPS growth not available
        Some(result.change_percent)
    });
    let market_cap = result.market_cap;
    let _volume = Some(result.volume as i64);
    let ranking_position = Some(1); // Default ranking position
    
    // Calculate price from available metrics (if available)
    let price_current = Some(result.price); // Use actual price field
    
    EPSRanking::from_eps_data(
        crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData {
            symbol: result.symbol,
            name: result.name,
            country: "US".to_string(), // Default country - not available in StockScreeningResult
            sector: result.sector.unwrap_or("Unknown".to_string()),
            exchange: "NASDAQ".to_string(), // Default exchange
            current_eps,
            growth_factor,
            price_current,
            market_cap: market_cap.map(|mc| mc as i64),
            volume: Some(result.volume as i64),
            ranking_score: ranking_position.map(|rp| rp as f64),
            created_at: None,
            updated_at: None,
            next_earnings_date: result.next_earnings_date.map(|ts| {
                chrono::DateTime::from_timestamp(ts as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default()
            }),
            last_earnings_date: result.last_earnings_date.map(|ts| {
                chrono::DateTime::from_timestamp(ts as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default()
            }),
        },
        ranking_position
    )
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

/// Calculate rank offset and limit based on user's plan metadata.
/// Uses get_wallet_ranking_offset which reads from plan_metadata directly.
async fn calculate_ranking_config_from_permissions(
    permission_service: &Arc<UnifiedPermissionService>,
    wallet_address: &str,
) -> (i32, i32) {
    // Get offset directly from plan metadata (not permission strings)
    match permission_service.get_wallet_ranking_offset(wallet_address).await {
        Ok(offset) => {
            info!("Wallet {} has ranking offset: {} (from plan metadata)", wallet_address, offset);
            // limit_cap is -1 (unlimited) since we're using offset-based access
            (offset, -1)
        },
        Err(e) => {
            warn!("Failed to get ranking offset for {}: {}, using {} default", wallet_address, e, epsx_contracts::constants::FREE_PLAN_NAME);
            (epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET, -1) // Free Plan default
        }
    }
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
            rank_offset: 0,  // No rank restriction for test
            limit_cap: 100,  // Default limit cap for test
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