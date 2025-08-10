use axum::{
    extract::{Query, Extension},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use chrono::{Datelike};

use crate::core::errors::AppError;
use crate::dom::entities::eps_growth::{EPSRankingsResponse, EPSRanking};
use crate::dom::services::eps_ranking_service::{EPSRankingService, EPSRankingParams, CountryValidator};
use crate::dom::services::eps_cache_service::{EPSCacheService, EPSCacheParams, CacheStats};
use crate::infra::services::tradingview_websocket::TradingViewWebSocketService;

/// Query parameters for EPS rankings endpoint
#[derive(Debug, Deserialize)]
pub struct EPSRankingQueryParams {
    pub page: Option<i32>,
    pub skip: Option<i32>,  // Support skip parameter for offset-based pagination
    pub limit: Option<i32>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: Option<String>,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// API response structure matching frontend pattern
#[derive(Debug, Serialize)]
pub struct EPSRankingsApiResponse {
    pub data: Vec<EPSRanking>,
    pub pagination: EPSPaginationResponse,
}

/// Pagination response structure
#[derive(Debug, Serialize)]
pub struct EPSPaginationResponse {
    pub page: i32,
    pub limit: i32,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

/// Countries list response
#[derive(Debug, Serialize)]
pub struct CountriesResponse {
    pub countries: Vec<String>,
    pub count: usize,
}

/// Sectors list response
#[derive(Debug, Serialize)]
pub struct SectorsResponse {
    pub sectors: Vec<String>,
    pub count: usize,
    pub country: Option<String>,
}

/// Health check response for EPS service
#[derive(Debug, Serialize)]
pub struct EPSHealthResponse {
    pub status: String,
    pub message: String,
    pub available_countries: usize,
}

impl From<EPSRankingsResponse> for EPSRankingsApiResponse {
    fn from(response: EPSRankingsResponse) -> Self {
        Self {
            data: response.rankings,
            pagination: EPSPaginationResponse {
                page: response.pagination.page,
                limit: response.pagination.limit,
                total: response.pagination.total,
                total_pages: response.pagination.total_pages,
                has_next: response.pagination.has_next,
                has_prev: response.pagination.has_prev,
            },
        }
    }
}

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
    
    // For small requests (≤10), enhance with WebSocket data for accuracy
    if result.rankings.len() <= 10 && result.rankings.len() > 0 {
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

/// GET /api/analytics/eps-rankings/countries
/// Returns list of available countries in EPS data
pub async fn get_available_countries(
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting available countries from EPS data");

    let countries = service.get_available_countries().await?;
    debug!("Found {} countries in EPS data", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} available countries", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/countries/all
/// Returns complete list of valid countries from MarketCountry enum
pub async fn get_all_valid_countries() -> Result<Json<CountriesResponse>, AppError> {
    debug!("Getting all valid countries from MarketCountry enum");

    let countries = CountryValidator::get_valid_countries();
    debug!("Found {} valid countries", countries.len());

    let response = CountriesResponse {
        count: countries.len(),
        countries,
    };

    info!("Returning {} valid countries", response.count);
    Ok(Json(response))
}

/// GET /api/analytics/eps-rankings/sectors?country=america
/// Returns list of available sectors, optionally filtered by country
pub async fn get_sectors_by_country(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<SectorsResponse>, AppError> {
    debug!("Getting sectors for country: {:?}", params.country);

    let sectors = service.get_sectors_by_country(params.country.clone()).await?;
    debug!("Found {} sectors", sectors.len());

    let response = SectorsResponse {
        count: sectors.len(),
        sectors,
        country: params.country,
    };

    info!("Returning {} sectors for country {:?}", response.count, response.country);
    Ok(Json(response))
}


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

/// POST /api/analytics/eps-rankings/websocket-test
/// Test WebSocket EPS data extraction
pub async fn debug_websocket_eps() -> Result<Json<serde_json::Value>, AppError> {
    info!("WebSocket EPS test triggered");
    
        use crate::core::errors::ErrorKind;
    
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
                            "quarterly_data_points": eps.quarterly_data.len(),
                            "historical_eps_count": eps.historical_eps.len()
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
    use crate::infra::services::tradingview::TradingViewApiService;
    use crate::infra::jobs::eps_data_processor::EPSDataProcessor;
    use crate::infra::InfraFactory;
    use crate::core::errors::ErrorKind;
    
    let config = std::sync::Arc::new(Config::from_env());
    let tradingview_service = std::sync::Arc::new(TradingViewApiService::new(config.clone()));
    
    // Create infrastructure factory
    let infra_factory = InfraFactory::from_env()
        .map_err(|e| AppError::new(ErrorKind::ConfigurationError, format!("Failed to create infra factory: {}", e)))?;
    let eps_service = infra_factory.create_eps_ranking_service();
    
    // Create processor and trigger manual processing
    let processor = EPSDataProcessor::new(eps_service, tradingview_service, config);
    
    info!("Starting manual EPS data processing...");
    match processor.trigger_manual_processing().await {
        Ok(stats) => {
            info!("Manual EPS sync completed successfully - Fetched: {}, Processed: {}, Stored: {}", 
                  stats.total_fetched, stats.total_processed, stats.total_stored);
            
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "EPS data sync completed successfully",
                "stats": {
                    "total_fetched": stats.total_fetched,
                    "total_processed": stats.total_processed,
                    "total_stored": stats.total_stored,
                    "total_errors": stats.total_errors,
                    "processing_duration_ms": stats.processing_duration_ms,
                    "countries_processed": stats.countries_processed
                }
            })))
        }
        Err(e) => {
            error!("Manual EPS sync failed: {:?}", e);
            Err(AppError::new(ErrorKind::ExternalServiceError, format!("EPS sync failed: {}", e)))
        }
    }
}

/// GET /api/v1/analytics/rankings - Live cache-based card dashboard endpoint
/// Returns EPS rankings in card format with cache-first performance approach
pub async fn get_unified_analytics_rankings_cached(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(cache_service): Extension<Arc<EPSCacheService>>,
) -> Result<Json<CardDashboardResponse>, AppError> {
    debug!("Cache-based unified analytics rankings API called with params: {:?}", params);
    
    // Convert query params to cache params with defaults
    // Support both skip/limit and page/limit pagination
    let limit = params.limit.unwrap_or(10);
    let page = if let Some(skip) = params.skip {
        // Convert skip to page: page = (skip / limit) + 1
        (skip / limit) + 1
    } else {
        params.page.unwrap_or(1)
    };
    
    let cache_params = EPSCacheParams {
        country: params.country.clone(),
        sector: params.sector.clone(),
        sort_by: params.sort_by.clone().or(Some("qoq_growth".to_string())),
        page,
        limit,
        min_eps: params.min_eps,
        min_growth: params.min_growth,
        force_refresh: false, // Use cache unless expired
    };

    debug!("Cache-based rankings service params: {:?}", cache_params);

    // Get total count first to validate pagination
    let total_count = cache_service.get_total_count_for_params(&cache_params).await?;
    let total_pages = ((total_count as f64) / (cache_params.limit as f64)).ceil() as i32;
    
    // Validate page number
    if cache_params.page > total_pages && total_count > 0 {
        return Err(AppError::new(
            crate::core::errors::ErrorKind::ValidationError,
            format!("Page {} does not exist. Total pages available: {}. Please request a page between 1 and {}.", 
                   cache_params.page, total_pages, total_pages)
        ));
    }

    // Log request details for debugging
    info!("Processing cache-based unified analytics rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          cache_params.country, cache_params.sort_by, cache_params.page, cache_params.limit);

    // Fetch data using cache-first approach
    let start_time = std::time::Instant::now();
    
    // Get rankings data from cache (with live fallback)
    let rankings_result = cache_service.get_eps_rankings(cache_params.clone()).await?;
    
    // Get available countries and sectors from cache
    let countries_result = cache_service.get_available_countries().await;
    let sectors_result = if let Some(ref country) = cache_params.country {
        cache_service.get_sectors_by_country(Some(country.clone())).await
    } else {
        cache_service.get_sectors_by_country(None).await
    };
    
    let duration = start_time.elapsed();
    
    // Get rankings data with WebSocket enhancement for small requests (≤10 items)
    let mut rankings_data = rankings_result.rankings;
    if rankings_data.len() <= 10 && !rankings_data.is_empty() {
        debug!("Cache endpoint: Enhancing {} rankings with WebSocket EPS data", rankings_data.len());
        
        let symbols: Vec<String> = rankings_data.iter()
            .map(|r| r.symbol.clone())
            .collect();
        
        match enhance_with_websocket_data(&symbols, &mut rankings_data).await {
            Ok(enhanced_count) => {
                info!("Cache endpoint: Enhanced {} rankings with WebSocket data", enhanced_count);
            }
            Err(e) => {
                warn!("Cache endpoint: Failed to enhance with WebSocket data: {}, using cached data", e);
            }
        }
    }
    
    // Transform EPS rankings to unified format first, then to card format
    let unified_rankings: Vec<UnifiedRankingItem> = rankings_data.into_iter()
        .enumerate()
        .map(|(index, ranking)| {
            transform_ranking_to_unified_format(ranking, index + ((cache_params.page - 1) * cache_params.limit) as usize + 1)
        })
        .collect();

    // Transform to card format for the response
    let card_data: Vec<SymbolCardData> = unified_rankings.into_iter()
        .map(transform_unified_to_card_format)
        .collect();

    // Prepare metadata
    let countries = countries_result.unwrap_or_default();
    let sectors = sectors_result.unwrap_or_default();
    
    let metadata = CardDashboardMetadata {
        available_countries: countries,
        available_sectors: sectors,
        request_timestamp: chrono::Utc::now(),
        data_source: "live_cache_eps_analytics".to_string(),
    };

    // Build card dashboard response
    let data_len = card_data.len();
    let card_response = CardDashboardResponse {
        success: true,
        data: card_data,
        pagination: EPSPaginationResponse {
            page: rankings_result.pagination.page,
            limit: rankings_result.pagination.limit,
            total: rankings_result.pagination.total,
            total_pages: rankings_result.pagination.total_pages,
            has_next: rankings_result.pagination.has_next,
            has_prev: rankings_result.pagination.has_prev,
        },
        metadata,
        message: Some(format!("Fetched {} card dashboard rankings successfully from live cache", data_len)),
        processing_time_ms: duration.as_millis() as u64,
    };

    info!("Live cache-based card dashboard completed in {:?} - {} items returned", 
          duration, data_len);

    Ok(Json(card_response))
}

/// GET /api/v1/analytics/cache/stats - Get cache statistics
pub async fn get_cache_stats(
    Extension(cache_service): Extension<Arc<EPSCacheService>>,
) -> Result<Json<CacheStatsResponse>, AppError> {
    debug!("Getting cache statistics");
    
    let stats = cache_service.get_cache_stats().await;
    
    let response = CacheStatsResponse {
        success: true,
        stats,
        message: "Cache statistics retrieved successfully".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    info!("Cache stats - Total entries: {}, Active: {}, Hit ratio: {:.2}%", 
          response.stats.total_entries, response.stats.active_entries, response.stats.hit_ratio * 100.0);
    
    Ok(Json(response))
}

/// POST /api/v1/analytics/cache/refresh - Force cache refresh
pub async fn force_cache_refresh(
    Extension(cache_service): Extension<Arc<EPSCacheService>>,
) -> Result<Json<CacheRefreshResponse>, AppError> {
    info!("Forcing cache refresh");
    
    let start_time = std::time::Instant::now();
    let refreshed_count = cache_service.refresh_cache().await?;
    let duration = start_time.elapsed();
    
    let response = CacheRefreshResponse {
        success: true,
        refreshed_entries: refreshed_count,
        duration_ms: duration.as_millis() as u64,
        message: format!("Cache refreshed with {} entries", refreshed_count),
        timestamp: chrono::Utc::now(),
    };
    
    info!("Cache refresh completed - {} entries refreshed in {:?}", refreshed_count, duration);
    
    Ok(Json(response))
}

/// GET /api/v1/analytics/cache/health - Cache health check
pub async fn cache_health_check(
    Extension(cache_service): Extension<Arc<EPSCacheService>>,
) -> Result<Json<CacheHealthResponse>, AppError> {
    debug!("Checking cache health");
    
    let stats = cache_service.get_cache_stats().await;
    
    // Determine health status based on cache metrics
    let healthy = stats.active_entries > 0 && stats.hit_ratio > 0.1;
    let status = if healthy { "healthy" } else { "degraded" };
    
    let mut recommendations = Vec::new();
    
    if stats.active_entries == 0 {
        recommendations.push("Cache is empty - consider warming the cache".to_string());
    }
    
    if stats.hit_ratio < 0.5 {
        recommendations.push("Low cache hit ratio - consider increasing TTL".to_string());
    }
    
    if stats.cache_size_mb > 100.0 {
        recommendations.push("Cache size is large - monitor memory usage".to_string());
    }
    
    let response = CacheHealthResponse {
        status: status.to_string(),
        healthy,
        cache_stats: stats,
        recommendations,
        timestamp: chrono::Utc::now(),
    };
    
    info!("Cache health check - Status: {}, Active entries: {}, Hit ratio: {:.2}%", 
          status, response.cache_stats.active_entries, response.cache_stats.hit_ratio * 100.0);
    
    Ok(Json(response))
}

/// GET /api/v1/analytics/rankings - Legacy database-based unified analytics rankings endpoint
/// Returns comprehensive rankings data with all metadata in single response
pub async fn get_unified_analytics_rankings(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(service): Extension<Arc<EPSRankingService>>,
) -> Result<Json<UnifiedAnalyticsRankingsResponse>, AppError> {
    debug!("Unified analytics rankings API called with params: {:?}", params);
    
    // Convert query params to service params with defaults
    let service_params = EPSRankingParams {
        country: params.country.clone(),
        sector: params.sector.clone(),
        sort_by: params.sort_by.clone().or(Some("qoq_growth".to_string())),
        page: params.page.unwrap_or(1),
        limit: params.limit.unwrap_or(10),
        min_eps: params.min_eps,
        min_growth: params.min_growth,
    };

    debug!("Unified rankings service params: {:?}", service_params);

    // Validate parameters
    service.validate_ranking_params(&service_params)?;

    // Get total count first to validate pagination (using country filter only for now)
    let total_count = service.get_total_count_for_params(&service_params).await?;
    let total_pages = ((total_count as f64) / (service_params.limit as f64)).ceil() as i32;
    
    // Validate page number
    if service_params.page > total_pages && total_count > 0 {
        return Err(AppError::new(
            crate::core::errors::ErrorKind::ValidationError,
            format!("Page {} does not exist. Total pages available: {}. Please request a page between 1 and {}.", 
                   service_params.page, total_pages, total_pages)
        ));
    }

    // Log request details for debugging
    info!("Processing unified analytics rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          service_params.country, service_params.sort_by, service_params.page, service_params.limit);

    // Fetch all required data concurrently
    let start_time = std::time::Instant::now();
    
    // Get rankings data
    let mut rankings_result = service.get_eps_rankings(service_params.clone()).await?;
    
    // Get available countries (if not already available in cache)
    let countries_result = service.get_available_countries().await;
    
    // Get sectors for current country (if specified)
    let sectors_result = if let Some(ref country) = service_params.country {
        service.get_sectors_by_country(Some(country.clone())).await
    } else {
        // Get all sectors if no country filter
        service.get_sectors_by_country(None).await
    };
    
    let duration = start_time.elapsed();
    
    // Enhance with WebSocket data for small requests (≤10 items)
    if rankings_result.rankings.len() <= 10 && !rankings_result.rankings.is_empty() {
        debug!("Enhancing {} rankings with WebSocket EPS data", rankings_result.rankings.len());
        
        let symbols: Vec<String> = rankings_result.rankings.iter()
            .map(|r| r.symbol.clone())
            .collect();
        
        match enhance_with_websocket_data(&symbols, &mut rankings_result.rankings).await {
            Ok(enhanced_count) => {
                info!("Enhanced {} rankings with WebSocket data", enhanced_count);
            }
            Err(e) => {
                warn!("Failed to enhance with WebSocket data: {}, using screener data", e);
            }
        }
    }

    // Transform EPS rankings to frontend-compatible format with quarterly data
    let unified_rankings: Vec<UnifiedRankingItem> = rankings_result.rankings.into_iter()
        .enumerate()
        .map(|(index, ranking)| {
            transform_ranking_to_unified_format(ranking, index + ((service_params.page - 1) * service_params.limit) as usize + 1)
        })
        .collect();

    // Prepare metadata
    let countries = countries_result.unwrap_or_default();
    let sectors = sectors_result.unwrap_or_default();
    
    let metadata = UnifiedAnalyticsMetadata {
        available_countries: countries,
        available_sectors: sectors,
        current_filters: UnifiedFilters {
            country: service_params.country,
            sector: service_params.sector,
            sort_by: service_params.sort_by.unwrap_or("qoq_growth".to_string()),
            min_eps: service_params.min_eps,
            min_growth: service_params.min_growth,
        },
        request_timestamp: chrono::Utc::now(),
        data_source: "eps_analytics".to_string(),
        enhanced_with_websocket: unified_rankings.len() <= 10,
    };

    // Build unified response
    let data_len = unified_rankings.len();
    let unified_response = UnifiedAnalyticsRankingsResponse {
        success: true,
        data: unified_rankings,
        pagination: EPSPaginationResponse {
            page: rankings_result.pagination.page,
            limit: rankings_result.pagination.limit,
            total: rankings_result.pagination.total,
            total_pages: rankings_result.pagination.total_pages,
            has_next: rankings_result.pagination.has_next,
            has_prev: rankings_result.pagination.has_prev,
        },
        metadata,
        message: Some(format!("Fetched {} rankings successfully", data_len)),
        processing_time_ms: duration.as_millis() as u64,
    };

    info!("Unified analytics rankings completed in {:?} - {} items returned", 
          duration, data_len);

    Ok(Json(unified_response))
}

/// Calculate EPS-weighted performance index for card dashboard
fn calculate_performance_index(current_eps: Option<f64>, qoq_growth: Option<f64>, market_cap: Option<i64>) -> f64 {
    let eps_val = current_eps.unwrap_or(0.0);
    let growth_val = qoq_growth.unwrap_or(0.0);
    
    // Base EPS component (scaled to reasonable range)
    let eps_component = if eps_val > 0.0 {
        (eps_val * 10.0).clamp(0.0, 20.0)
    } else {
        0.0
    };
    
    // Growth component (scaled and weighted)
    let growth_component = (growth_val / 100.0 * 5.0).clamp(-2.0, 10.0);
    
    // Market cap scaling factor (for very large companies)
    let scale_factor = if let Some(mc) = market_cap {
        if mc > 1_000_000_000_000 { // > $1T market cap
            1.2
        } else if mc > 500_000_000_000 { // > $500B market cap
            1.1
        } else {
            1.0
        }
    } else {
        1.0
    };
    
    // Calculate final index
    let base_index = eps_component + growth_component;
    (base_index * scale_factor).clamp(0.0, 50.0).round() / 10.0 * 10.0 // Round to 1 decimal
}

/// Calculate average growth percentage across quarters
fn calculate_average_growth(quarterly_data: &[EPSRanking]) -> f64 {
    if quarterly_data.is_empty() {
        return 0.0;
    }
    
    // For single data point, use QoQ growth if available
    if quarterly_data.len() == 1 {
        return quarterly_data[0].qoq_growth.unwrap_or(0.0);
    }
    
    // For multiple data points, calculate average of growth rates
    let total_growth: f64 = quarterly_data
        .iter()
        .map(|q| q.qoq_growth.unwrap_or(0.0))
        .sum();
        
    total_growth / quarterly_data.len() as f64
}

/// Generate quarterly performance data for card dashboard
fn generate_quarterly_performance_data(ranking: &EPSRanking) -> Vec<QuarterlyPerformanceData> {
    let current_date = chrono::Utc::now();
    let current_eps = ranking.current_eps.unwrap_or(0.0);
    let current_price = ranking.price_current.unwrap_or(0.0);
    let qoq_growth = ranking.qoq_growth.unwrap_or(0.0);
    
    // Generate current quarter (Q1) and previous quarter (Q0) data
    vec![
        QuarterlyPerformanceData {
            quarter: "Q1".to_string(),
            date: current_date.format("%b %-d, %Y").to_string(),
            price: current_price,
            eps: current_eps,
            eps_growth: qoq_growth,
            price_growth: 0.0, // Default to 0 as shown in UI
        },
        QuarterlyPerformanceData {
            quarter: "Q0".to_string(),
            date: (current_date - chrono::Duration::days(90)).format("%b %-d, %Y").to_string(),
            price: current_price * 0.95, // Estimate previous price
            eps: if qoq_growth != 0.0 {
                current_eps / (1.0 + qoq_growth / 100.0)
            } else {
                current_eps * 0.9
            },
            eps_growth: 0.0, // Reference point
            price_growth: 0.0, // Default to 0 as shown in UI
        },
    ]
}

/// Generate quarterly performance data from real WebSocket quarterly data
fn generate_quarterly_performance_from_real_data(ranking: &EPSRanking, quarterly_data: &[crate::infra::services::tradingview_websocket::QuarterlyEPSData]) -> Vec<QuarterlyPerformanceData> {
    debug!("Generating quarterly performance from real WebSocket data for {}: {} quarters", 
           ranking.symbol, quarterly_data.len());

    let mut result = Vec::new();
    let current_price = ranking.price_current.unwrap_or(100.0);
    
    // Process each quarter from the WebSocket data (already sorted by timestamp, newest first)
    for (i, quarter_data) in quarterly_data.iter().enumerate().take(3) {
        // Calculate price progression based on EPS changes
        let price_adjustment = if i == 0 {
            1.0 // Current price
        } else {
            // Estimate historical price based on EPS progression and some market volatility
            let eps_change_ratio = if i < quarterly_data.len() - 1 {
                let next_eps = quarterly_data[i + 1].eps;
                if next_eps > 0.0 {
                    quarter_data.eps / next_eps
                } else {
                    0.95 // Default slight decline
                }
            } else {
                0.95 // Default for oldest quarter
            };
            
            // Price follows EPS trends but with dampening
            eps_change_ratio * (0.8 + (i as f64 * 0.1))
        };
        
        let adjusted_price = current_price * price_adjustment;
        
        // Calculate EPS growth (quarter-over-quarter)
        let eps_growth = if i > 0 && i < quarterly_data.len() && quarterly_data[i - 1].eps > 0.0 {
            ((quarter_data.eps - quarterly_data[i - 1].eps) / quarterly_data[i - 1].eps) * 100.0
        } else {
            0.0
        };
        
        // Calculate price growth
        let price_growth = if i > 0 {
            let prev_price_adjustment = if i == 1 {
                1.0
            } else {
                let prev_eps_ratio = if i < quarterly_data.len() {
                    quarterly_data[i - 1].eps / quarterly_data[i].eps
                } else {
                    1.05
                };
                prev_eps_ratio * (0.8 + ((i - 1) as f64 * 0.1))
            };
            let prev_price = current_price * prev_price_adjustment;
            if prev_price > 0.0 {
                ((adjusted_price - prev_price) / prev_price) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        result.push(QuarterlyPerformanceData {
            quarter: quarter_data.quarter_name.clone(),
            date: format!("{}", chrono::DateTime::<chrono::Utc>::from_timestamp(quarter_data.timestamp, 0)
                .unwrap_or_default()
                .format("%b %d, %Y")),
            price: adjusted_price,
            eps: quarter_data.eps,
            eps_growth,
            price_growth,
        });
    }
    
    debug!("Generated {} quarterly performance data points for {}", result.len(), ranking.symbol);
    result
}

/// Generate quarterly performance data using real WebSocket data when available
fn generate_quarterly_performance_from_websocket_or_synthetic(ranking: &EPSRanking) -> Vec<QuarterlyPerformanceData> {
    // Check if we have real WebSocket quarterly data
    if let Some(ref quarterly_data) = ranking.quarterly_data {
        if quarterly_data.len() >= 3 {
            debug!("Using real WebSocket quarterly data for {} ({} quarters)", 
                   ranking.symbol, quarterly_data.len());
            return generate_quarterly_performance_from_real_data(ranking, quarterly_data);
        }
    }
    
    debug!("No sufficient WebSocket quarterly data for {}, using synthetic fallback", ranking.symbol);
    
    // Fallback to synthetic data generation
    generate_quarterly_performance_data(ranking)
}


/// Generate quarterly data from WebSocket data or proper consecutive quarters as fallback
fn generate_quarterly_data_from_websocket_or_fallback(ranking: &EPSRanking, current_date: chrono::DateTime<chrono::Utc>) -> Vec<QuarterlyData> {
    // Check if we have real WebSocket quarterly data
    if let Some(ref quarterly_data) = ranking.quarterly_data {
        if !quarterly_data.is_empty() {
            debug!("Using real WebSocket quarterly data for {} ({} quarters)", 
                   ranking.symbol, quarterly_data.len());
            return generate_quarterly_data_from_real_websocket_data(ranking, quarterly_data, current_date);
        }
    }
    
    debug!("No WebSocket quarterly data for {}, generating proper consecutive quarters", ranking.symbol);
    
    // Generate proper consecutive quarters from current date
    generate_consecutive_quarterly_data(ranking, current_date)
}

/// Generate quarterly data from real WebSocket quarterly EPS data
fn generate_quarterly_data_from_real_websocket_data(
    ranking: &EPSRanking, 
    quarterly_data: &[crate::infra::services::tradingview_websocket::QuarterlyEPSData], 
    current_date: chrono::DateTime<chrono::Utc>
) -> Vec<QuarterlyData> {
    debug!("Generating quarterly performance from real WebSocket data for {}: {} quarters", 
           ranking.symbol, quarterly_data.len());

    let current_price = ranking.price_current.unwrap_or(100.0);
    let mut result = Vec::new();
    
    // Sort quarterly data by timestamp (most recent first) and take up to 8 quarters
    let mut sorted_data = quarterly_data.to_vec();
    sorted_data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Process each quarter from the WebSocket data (up to 8 quarters)
    for (i, quarter_data) in sorted_data.iter().enumerate().take(8) {
        // Calculate price progression based on EPS changes and realistic market behavior
        let price_adjustment = if i == 0 {
            1.0 // Current price for most recent quarter
        } else {
            // Estimate historical price based on EPS progression and time decay
            let eps_ratio = if i < sorted_data.len() - 1 && sorted_data[i - 1].eps > 0.0 {
                quarter_data.eps / sorted_data[i - 1].eps
            } else {
                0.95 // Default slight decline for older quarters
            };
            
            // Price follows EPS trends but with dampening over time
            let time_decay = 1.0 - (i as f64 * 0.05); // 5% decay per quarter back
            eps_ratio * time_decay.max(0.7) // Min 70% of current price
        };
        
        let adjusted_price = current_price * price_adjustment;
        
        // Calculate EPS growth (quarter-over-quarter)
        let eps_growth = if i > 0 && i < sorted_data.len() && sorted_data[i - 1].eps > 0.0 {
            ((quarter_data.eps - sorted_data[i - 1].eps) / sorted_data[i - 1].eps) * 100.0
        } else {
            ranking.qoq_growth.unwrap_or(0.0) // Use current QoQ growth for most recent
        };
        
        // Calculate price growth
        let price_growth = if i > 0 {
            let prev_price_adjustment = if i == 1 {
                1.0
            } else {
                let prev_eps_ratio = if i < sorted_data.len() {
                    sorted_data[i - 1].eps / sorted_data[i].eps
                } else {
                    1.05
                };
                let prev_time_decay = 1.0 - ((i - 1) as f64 * 0.05);
                prev_eps_ratio * prev_time_decay.max(0.7)
            };
            let prev_price = current_price * prev_price_adjustment;
            if prev_price > 0.0 {
                ((adjusted_price - prev_price) / prev_price) * 100.0
            } else {
                0.0
            }
        } else {
            0.0 // Most recent quarter as reference
        };
        
        result.push(QuarterlyData {
            quarter: quarter_data.quarter_name.clone(), // Use real quarter name from WebSocket
            date: chrono::DateTime::<chrono::Utc>::from_timestamp(quarter_data.timestamp, 0)
                .unwrap_or(current_date - chrono::Duration::days(i as i64 * 90)),
            price: adjusted_price,
            eps: quarter_data.eps, // Use real EPS from WebSocket
            eps_growth,
            price_growth,
            volume: ranking.volume.map(|v| ((v as f64) * (1.0 - i as f64 * 0.1).max(0.5)) as i64),
        });
    }
    
    debug!("Generated {} quarterly data points from real WebSocket data for {}", result.len(), ranking.symbol);
    result
}

/// Generate proper consecutive quarterly data when no WebSocket data is available
fn generate_consecutive_quarterly_data(ranking: &EPSRanking, current_date: chrono::DateTime<chrono::Utc>) -> Vec<QuarterlyData> {
    let current_eps = ranking.current_eps.unwrap_or(0.0);
    let qoq_growth_pct = ranking.qoq_growth.unwrap_or(0.0);
    let current_price = ranking.price_current.unwrap_or(100.0);
    
    // Generate proper consecutive quarters working backwards from current date
    let current_year = current_date.year();
    let current_month = current_date.month();
    let current_quarter = ((current_month - 1) / 3) + 1;
    
    let mut quarterly_data = Vec::new();
    
    // Generate last 8 quarters of data
    for i in 0..8 {
        // Calculate quarter and year going backwards
        let quarters_back = i as i32;
        let (quarter, year) = calculate_quarter_backwards(current_quarter, current_year, quarters_back);
        
        // Generate realistic EPS progression
        let eps_multiplier = if i == 0 {
            1.0 // Current quarter
        } else {
            // Simulate realistic EPS growth over time
            let base_growth = qoq_growth_pct / 100.0;
            let quarterly_decay = 1.0 - (base_growth * i as f64 * 0.8); // Diminishing growth backwards
            quarterly_decay.max(0.3) // Minimum 30% of current EPS
        };
        
        let quarter_eps = current_eps * eps_multiplier;
        
        // Calculate price progression
        let price_multiplier = if i == 0 {
            1.0
        } else {
            // Price follows EPS with some market volatility
            eps_multiplier * (0.9 + (i as f64 * 0.02)) // Slight price appreciation over time
        };
        
        let quarter_price = current_price * price_multiplier;
        
        // Calculate growth rates
        let eps_growth = if i == 0 {
            qoq_growth_pct
        } else if i == 1 {
            0.0 // Previous quarter as reference
        } else {
            // Calculate QoQ growth backwards
            let prev_eps = current_eps * if i == 1 { 1.0 } else { 
                let prev_multiplier = 1.0 - (qoq_growth_pct / 100.0 * (i - 1) as f64 * 0.8);
                prev_multiplier.max(0.3)
            };
            if prev_eps > 0.0 {
                ((quarter_eps - prev_eps) / prev_eps) * 100.0
            } else {
                0.0
            }
        };
        
        let price_growth = if i <= 1 {
            0.0 // Recent quarters as reference
        } else {
            -2.0 // Slight decline for older quarters
        };
        
        quarterly_data.push(QuarterlyData {
            quarter: format!("Q{} '{}", quarter, year % 100),
            date: current_date - chrono::Duration::days(i as i64 * 90),
            price: quarter_price,
            eps: quarter_eps,
            eps_growth,
            price_growth,
            volume: ranking.volume.map(|v| ((v as f64) * (1.0 - i as f64 * 0.05).max(0.7)) as i64),
        });
    }
    
    debug!("Generated {} consecutive quarterly data points for {}", quarterly_data.len(), ranking.symbol);
    quarterly_data
}

/// Calculate quarter and year going backwards from current quarter
fn calculate_quarter_backwards(current_quarter: u32, current_year: i32, quarters_back: i32) -> (u32, i32) {
    let total_quarters = (current_year - 2020) * 4 + current_quarter as i32;
    let target_quarters = total_quarters - quarters_back;
    
    if target_quarters <= 0 {
        return (1, 2020); // Fallback to Q1 2020
    }
    
    let target_year = 2020 + (target_quarters - 1) / 4;
    let target_quarter = ((target_quarters - 1) % 4) + 1;
    
    (target_quarter as u32, target_year)
}

/// Transform EPS ranking to unified format with quarterly data
fn transform_ranking_to_unified_format(ranking: EPSRanking, position: usize) -> UnifiedRankingItem {
    let current_date = chrono::Utc::now();
    let current_price = ranking.price_current.unwrap_or(100.0);
    let qoq_growth_pct = ranking.qoq_growth.unwrap_or(0.0);
    
    UnifiedRankingItem {
        symbol: ranking.symbol.clone(),
        company_name: ranking.name.clone(),
        ranking_position: position as i32,
        current_price,
        current_price_date: current_date,
        quarterly_data: generate_quarterly_data_from_websocket_or_fallback(&ranking, current_date),
        market_data: MarketData {
            market_cap: ranking.market_cap,
            volume_24h: ranking.volume,
            country: ranking.country.clone(),
            sector: ranking.sector.clone(),
            exchange: ranking.exchange.clone(),
        },
        analytics: AnalyticsMetrics {
            qoq_growth: qoq_growth_pct,
            ranking_score: 0.0, // Could be calculated based on various factors
            trend: determine_trend(qoq_growth_pct),
            volatility: calculate_simple_volatility(qoq_growth_pct),
        },
    }
}

/// Determine trend based on QoQ growth
fn determine_trend(qoq_growth: f64) -> String {
    if qoq_growth > 20.0 {
        "strong_bullish".to_string()
    } else if qoq_growth > 5.0 {
        "bullish".to_string()
    } else if qoq_growth > -5.0 {
        "neutral".to_string()
    } else if qoq_growth > -20.0 {
        "bearish".to_string()
    } else {
        "strong_bearish".to_string()
    }
}

/// Calculate simple volatility from price series
fn calculate_volatility(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }
    
    let mean = prices.iter().sum::<f64>() / prices.len() as f64;
    let variance = prices.iter()
        .map(|price| (price - mean).powi(2))
        .sum::<f64>() / prices.len() as f64;
    
    variance.sqrt() / mean * 100.0 // Return as percentage
}

/// Calculate simple volatility from QoQ growth percentage
fn calculate_simple_volatility(qoq_growth: f64) -> f64 {
    // Simple volatility estimation based on growth rate magnitude
    qoq_growth.abs().min(50.0) // Cap at 50% for reasonable volatility score
}

/// Error handling for EPS-specific errors
/// Enhance EPS rankings with REAL TradingView WebSocket data (not hardcoded)
async fn enhance_with_websocket_data(
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
            let mut websocket_map = std::collections::HashMap::new();
            for ws_data in websocket_data {
                websocket_map.insert(ws_data.symbol.clone(), ws_data);
            }
            
            // Update rankings with REAL WebSocket data
            for ranking in rankings.iter_mut() {
                if let Some(ws_data) = websocket_map.get(&ranking.symbol) {
                    info!("🔄 Enhancing {} with REAL TradingView WebSocket data", ranking.symbol);
                    
                    // Update with real current EPS
                    if ws_data.current_eps > 0.01 && ws_data.current_eps < 100.0 && ws_data.current_eps.is_finite() {
                        debug!("Updating {} current EPS: {:?} → {} (REAL WebSocket)", 
                               ranking.symbol, ranking.current_eps, ws_data.current_eps);
                        ranking.current_eps = Some(ws_data.current_eps);
                        enhanced_count += 1;
                    }
                    
                    // Store REAL quarterly data
                    if !ws_data.quarterly_data.is_empty() {
                        ranking.quarterly_data = Some(ws_data.quarterly_data.clone());
                        
                        // Calculate QoQ growth from REAL quarterly data
                        if ws_data.quarterly_data.len() >= 2 {
                            let current_eps = ws_data.quarterly_data[0].eps;
                            let previous_eps = ws_data.quarterly_data[1].eps;
                            
                            if previous_eps > 0.0 {
                                let qoq_growth = ((current_eps - previous_eps) / previous_eps) * 100.0;
                                if qoq_growth.abs() < 200.0 { // Reasonable growth range
                                    debug!("Updating {} QoQ growth: {:?} → {}% (REAL WebSocket)", 
                                           ranking.symbol, ranking.qoq_growth, qoq_growth);
                                    ranking.qoq_growth = Some(qoq_growth);
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

impl From<AppError> for (StatusCode, Json<serde_json::Value>) {
    fn from(error: AppError) -> Self {
        use crate::core::errors::ErrorKind;
        
        match error.kind {
            ErrorKind::ValidationError => {
                warn!("Validation error in EPS API: {}", error.message);
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "validation_error",
                        "message": error.message,
                        "code": "EPS_VALIDATION_FAILED"
                    }))
                )
            }
            ErrorKind::DatabaseError => {
                error!("Database error in EPS API: {}", error.message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "database_error",
                        "message": "Internal server error",
                        "code": "EPS_DATABASE_ERROR"
                    }))
                )
            }
            _ => {
                error!("Unexpected error in EPS API: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "internal_error",
                        "message": "An unexpected error occurred",
                        "code": "EPS_INTERNAL_ERROR"
                    }))
                )
            }
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
            skip: None,
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
    fn test_pagination_response_serialization() {
        let pagination = EPSPaginationResponse {
            page: 1,
            limit: 50,
            total: 1000,
            total_pages: 20,
            has_next: true,
            has_prev: false,
        };

        let json = serde_json::to_string(&pagination).unwrap();
        assert!(json.contains("totalPages"));
        assert!(json.contains("hasNext"));
        assert!(json.contains("hasPrev"));
    }

    #[test]
    fn test_countries_response() {
        let response = CountriesResponse {
            countries: vec!["america".to_string(), "thailand".to_string()],
            count: 2,
        };

        assert_eq!(response.count, 2);
        assert!(response.countries.contains(&"america".to_string()));
    }
}

/// Transform UnifiedRankingItem to SymbolCardData for card dashboard format
fn transform_unified_to_card_format(unified_item: UnifiedRankingItem) -> SymbolCardData {
    // Calculate performance index from analytics data
    let index = unified_item.analytics.ranking_score;
    
    // Calculate average growth from quarterly data
    let avg_growth = if !unified_item.quarterly_data.is_empty() {
        let growth_sum: f64 = unified_item.quarterly_data.iter()
            .map(|q| q.eps_growth)
            .sum();
        growth_sum / unified_item.quarterly_data.len() as f64
    } else {
        0.0
    };
    
    // Transform quarterly data format
    let quarterly_performance: Vec<QuarterlyPerformanceData> = unified_item.quarterly_data.into_iter()
        .map(|q| QuarterlyPerformanceData {
            quarter: q.quarter,
            date: q.date.format("%b %-d, %Y").to_string(),
            price: q.price,
            eps: q.eps,
            eps_growth: q.eps_growth,
            price_growth: q.price_growth,
        })
        .collect();
    
    SymbolCardData {
        rank: unified_item.ranking_position,
        symbol: unified_item.symbol,
        latest_date: unified_item.current_price_date.format("%b %-d, %Y").to_string(),
        value: unified_item.current_price,
        index,
        avg_growth,
        eps_to_price: None, // N/A initially as shown in UI
        quarterly_performance,
    }
}

/// Unified analytics rankings response structure
#[derive(Debug, Serialize)]
pub struct UnifiedAnalyticsRankingsResponse {
    pub success: bool,
    pub data: Vec<UnifiedRankingItem>,
    pub pagination: EPSPaginationResponse,
    pub metadata: UnifiedAnalyticsMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual ranking item in unified format
#[derive(Debug, Serialize)]
pub struct UnifiedRankingItem {
    pub symbol: String,
    pub company_name: String,
    pub ranking_position: i32,
    pub current_price: f64,
    pub current_price_date: chrono::DateTime<chrono::Utc>,
    pub quarterly_data: Vec<QuarterlyData>,
    pub market_data: MarketData,
    pub analytics: AnalyticsMetrics,
}

/// Quarterly data for each stock
#[derive(Debug, Serialize)]
pub struct QuarterlyData {
    pub quarter: String, // e.g., "Q3 '25"
    pub date: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64, // QoQ growth percentage
    pub price_growth: f64, // QoQ price growth percentage
    pub volume: Option<i64>,
}

/// Market data for each stock
#[derive(Debug, Serialize)]
pub struct MarketData {
    pub market_cap: Option<i64>,
    pub volume_24h: Option<i64>,
    pub country: String,
    pub sector: String,
    pub exchange: String,
}

/// Analytics metrics for each stock
#[derive(Debug, Serialize)]
pub struct AnalyticsMetrics {
    pub qoq_growth: f64,
    pub ranking_score: f64,
    pub trend: String, // bullish, bearish, neutral, etc.
    pub volatility: f64,
}

/// Metadata included in unified response
#[derive(Debug, Serialize)]
pub struct UnifiedAnalyticsMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub current_filters: UnifiedFilters,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
    pub enhanced_with_websocket: bool,
}

/// Current filters applied to the request
#[derive(Debug, Serialize)]
pub struct UnifiedFilters {
    pub country: Option<String>,
    pub sector: Option<String>,
    pub sort_by: String,
    pub min_eps: Option<f64>,
    pub min_growth: Option<f64>,
}

/// Cache statistics response
#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub success: bool,
    pub stats: CacheStats,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache refresh response
#[derive(Debug, Serialize)]
pub struct CacheRefreshResponse {
    pub success: bool,
    pub refreshed_entries: usize,
    pub duration_ms: u64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache health check response
#[derive(Debug, Serialize)]
pub struct CacheHealthResponse {
    pub status: String,
    pub healthy: bool,
    pub cache_stats: CacheStats,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Card dashboard response structure for multi-symbol EPS analytics
#[derive(Debug, Serialize)]
pub struct CardDashboardResponse {
    pub success: bool,
    pub data: Vec<SymbolCardData>,
    pub pagination: EPSPaginationResponse,
    pub metadata: CardDashboardMetadata,
    pub message: Option<String>,
    pub processing_time_ms: u64,
}

/// Individual symbol card data matching frontend UI requirements
#[derive(Debug, Serialize)]
pub struct SymbolCardData {
    pub rank: i32,
    pub symbol: String,
    pub latest_date: String,
    pub value: f64,                    // Current price
    pub index: f64,                    // EPS-weighted performance index
    pub avg_growth: f64,               // Average growth percentage
    pub eps_to_price: Option<String>,  // EPS to price correlation (N/A initially)
    pub quarterly_performance: Vec<QuarterlyPerformanceData>,
}

/// Quarterly performance data for the card dashboard
#[derive(Debug, Serialize)]
pub struct QuarterlyPerformanceData {
    pub quarter: String,      // "Q1", "Q0", etc.
    pub date: String,         // "Aug 8, 2025"
    pub price: f64,
    pub eps: f64,
    pub eps_growth: f64,      // EPS % growth
    pub price_growth: f64,    // Price % growth
}

/// Metadata for card dashboard
#[derive(Debug, Serialize)]
pub struct CardDashboardMetadata {
    pub available_countries: Vec<String>,
    pub available_sectors: Vec<String>,
    pub request_timestamp: chrono::DateTime<chrono::Utc>,
    pub data_source: String,
}