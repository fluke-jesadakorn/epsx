// Cache Management for EPS Analytics
// Focused module handling caching logic and cache-related endpoints

use axum::{
    extract::{Query, Extension},
    response::Json,
};
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{debug, info, warn};

use crate::core::errors::AppError;
use crate::dom::entities::eps_growth::EPSRanking;
use crate::dom::services::eps_cache_service::EPSCacheService;
use crate::dom::services::eps_ranking_service::EPSRankingService;
use crate::infra::cache::{Cache, CacheExt};
use crate::infra::services::tradingview::TradingViewService;
use super::{
    dto::*, 
    rankings::convert_screening_result_to_eps_ranking,
    enhancement::enhance_with_websocket_data,
    transform::{transform_ranking_to_unified_format, transform_unified_to_card_format},
    metadata::{get_available_countries_static, get_available_sectors_static}
};

/// GET /api/v1/analytics/rankings - Direct TradingView card dashboard endpoint with caching
/// Returns EPS rankings in card format with unified cache support (Redis/Memory)
pub async fn get_unified_analytics_rankings_cached(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(_eps_ranking_service): Extension<Arc<EPSRankingService>>,
    Extension(cache): Extension<Arc<dyn Cache>>,
) -> Result<Json<CardDashboardResponse>, AppError> {
    debug!("Direct TradingView analytics rankings API called with params: {:?}", params);
    
    // Convert query params to service params with defaults
    let limit = params.limit.unwrap_or(10);
    let page = params.page.unwrap_or(1).max(1); // Ensure page is at least 1
    let skip = (page - 1) * limit; // Convert page to skip internally
    
    // Generate cache key for this request
    let cache_key = generate_cache_key(&params);
    debug!("Generated cache key: {}", cache_key);
    
    // Check cache first (1-hour TTL)
    if let Ok(Some(cached_response)) = cache.get::<CardDashboardResponse>(&cache_key).await {
        info!("Cache hit for analytics rankings - returning cached data");
        return Ok(Json(cached_response));
    }
    
    debug!("Cache miss for analytics rankings - fetching fresh data");
    
    // Log request details for debugging
    info!("Processing direct TradingView analytics rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}", 
          params.country, params.sort_by, page, limit);

    // Fetch data using direct TradingView API calls
    let start_time = std::time::Instant::now();
    
    // Create TradingView service for direct API calls
    let config = match crate::config::Config::from_env() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            tracing::warn!("Failed to load config, using fallback: {:?}", e);
            Arc::new(get_fallback_config())
        }
    };
    let tradingview_service = crate::infra::services::tradingview::TradingViewApiService::new(config);
    
    // Get rankings data directly from TradingView
    let (screening_results, total_count) = tradingview_service
        .fetch_eps_growth_ranking(
            Some(skip),
            Some(limit),
            params.country.clone(),
            params.sector.clone(),
            params.sort_by.clone().or(Some("market_cap".to_string())),
        )
        .await
        .map_err(|e| AppError::new(
            crate::core::errors::ErrorKind::ExternalServiceError,
            format!("TradingView API error: {}", e)
        ))?;
    
    // Convert TradingView screening results to EPS rankings format
    let mut rankings_data: Vec<EPSRanking> = screening_results.into_iter()
        .map(|result| convert_screening_result_to_eps_ranking(result))
        .collect();
    
    // Get rankings data with WebSocket enhancement for small requests (≤20 items)  
    if rankings_data.len() <= 20 && !rankings_data.is_empty() {
        debug!("Direct endpoint: Enhancing {} rankings with WebSocket EPS data", rankings_data.len());
        
        let symbols: Vec<String> = rankings_data.iter()
            .map(|r| r.symbol.clone())
            .collect();
        
        match enhance_with_websocket_data(&symbols, &mut rankings_data).await {
            Ok(enhanced_count) => {
                info!("Direct endpoint: Enhanced {} rankings with WebSocket data", enhanced_count);
            }
            Err(e) => {
                warn!("Direct endpoint: Failed to enhance with WebSocket data: {}, using TradingView data", e);
            }
        }
    }
    
    // Transform EPS rankings to unified format first, then to card format
    let unified_rankings: Vec<super::dto::UnifiedRankingItem> = rankings_data.into_iter()
        .enumerate()
        .map(|(index, ranking)| {
            transform_ranking_to_unified_format(ranking, index + skip as usize + 1)
        })
        .collect();

    // Transform to card format for the response
    let card_data: Vec<SymbolCardData> = unified_rankings.into_iter()
        .map(transform_unified_to_card_format)
        .collect();

    // Calculate pagination metadata
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;
    let has_next = page < total_pages;
    let has_prev = page > 1;

    // Prepare metadata - use static data for now since we removed cache
    let metadata = CardDashboardMetadata {
        available_countries: get_available_countries_static(),
        available_sectors: get_available_sectors_static(),
        request_timestamp: chrono::Utc::now(),
        data_source: "live_tradingview_api".to_string(),
    };
    
    let duration = start_time.elapsed();

    // Build card dashboard response
    let data_len = card_data.len();
    let card_response = CardDashboardResponse {
        success: true,
        data: card_data,
        pagination: EPSPaginationResponse {
            page,
            limit,
            total: total_count as i64,
            total_pages,
            has_next,
            has_prev,
        },
        metadata,
        message: Some(format!("Fetched {} card dashboard rankings successfully from TradingView API", data_len)),
        processing_time_ms: duration.as_millis() as u64,
    };

    info!("Direct TradingView API card dashboard completed in {:?} - {} items returned", 
          duration, data_len);

    // Store response in cache with 1-hour TTL (3600 seconds)
    if let Err(e) = cache.set(&cache_key, &card_response, Some(3600)).await {
        warn!("Failed to store analytics rankings in cache: {}", e);
        // Don't fail the request if cache storage fails
    } else {
        debug!("Successfully cached analytics rankings with key: {}", cache_key);
    }

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

/// Generate cache key from query parameters for analytics rankings
pub fn generate_cache_key(params: &EPSRankingQueryParams) -> String {
    let mut hasher = DefaultHasher::new();
    
    // Hash relevant parameters
    params.country.hash(&mut hasher);
    params.sector.hash(&mut hasher);
    params.sort_by.hash(&mut hasher);
    params.page.unwrap_or(1).hash(&mut hasher);
    params.limit.unwrap_or(10).hash(&mut hasher);
    
    // Handle f64 fields by converting to strings (to avoid NaN hash issues)
    if let Some(min_eps) = params.min_eps {
        min_eps.to_string().hash(&mut hasher);
    }
    if let Some(min_growth) = params.min_growth {
        min_growth.to_string().hash(&mut hasher);
    }
    
    let hash = hasher.finish();
    format!("analytics:rankings:{:x}", hash)
}

/// Get fallback config when environment config fails
fn get_fallback_config() -> crate::config::Config {
    crate::config::Config {
        server: crate::config::ServerConfig {
            port: 8080,
            host: "127.0.0.1".to_string(),
            bind_address: "0.0.0.0".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
            admin_frontend_url: "http://localhost:3001".to_string(),
            environment: "development".to_string(),
        },
        database: crate::config::DatabaseConfig {
            url: "postgresql://localhost/epsx".to_string(),
        },
        auth: crate::config::AuthConfig {
            jwt_secret_main: "default-jwt-secret".to_string(),
            jwt_secret: "default-jwt-secret".to_string(),
            cookie_signing_key: None,
            cookie_encryption_key: None,
            firebase_project_id: None,
            backend_url: "http://localhost:8080".to_string(),
            oidc_issuer: "http://localhost:8080".to_string(),
        },
        payment: crate::config::PaymentConfig {
            musepay_partner_id: None,
            musepay_private_key: None,
            webhook_url: None,
            supported_currencies: vec!["USD".to_string(), "EUR".to_string()],
            default_currency: "USD".to_string(),
            default_checkout_url_template: "https://localhost:3000/checkout/{}".to_string(),
        },
        email: crate::config::EmailConfig {
            from_email: "noreply@localhost".to_string(),
            from_name: "EPSX".to_string(),
            sendgrid_api_key: "".to_string(),
        },
        branding: crate::config::BrandingConfig {
            platform_name: "EPSX".to_string(),
            welcome_message_template: "Welcome to EPSX".to_string(),
            dashboard_url: "http://localhost:3000".to_string(),
            support_email: "support@localhost".to_string(),
        },
        external_services: crate::config::ExternalServicesConfig {
            tradingview: crate::config::TradingViewConfig {
                websocket_url: "wss://data.tradingview.com".to_string(),
                api_base_url: "https://scanner.tradingview.com".to_string(),
                timeout_seconds: 30,
                http_timeout_seconds: 30,
            },
            sendgrid_api_key: None,
            qr_code: crate::config::QrCodeConfig {
                enabled: false,
                base_url: "http://localhost:8080".to_string(),
                logo_url: None,
                api_base_url: "http://localhost:8080".to_string(),
                default_size: 256,
            },
        },
        rate_limiting: crate::config::RateLimitingConfig {
            default_per_minute: 60,
            endpoint_specific: std::collections::HashMap::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let params = EPSRankingQueryParams {
            page: Some(1),
            limit: Some(10),
            country: Some("america".to_string()),
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        let cache_key = generate_cache_key(&params);
        assert!(cache_key.starts_with("analytics:rankings:"));
        assert!(cache_key.len() > 20); // Should be a hex hash

        // Same params should generate same key
        let cache_key2 = generate_cache_key(&params);
        assert_eq!(cache_key, cache_key2);
    }

    #[test] 
    fn test_fallback_config() {
        let config = get_fallback_config();
        assert_eq!(config.server.port, 8080);
        assert!(!config.external_services.tradingview.api_base_url.is_empty());
    }
}