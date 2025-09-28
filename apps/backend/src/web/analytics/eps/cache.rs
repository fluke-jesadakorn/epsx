// Cache Management for EPS Analytics
// Focused module handling caching logic and cache-related endpoints

use axum::{ extract::{ Query, Extension }, response::Json };
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };
use tracing::{ debug, info, warn };

use crate::core::errors::AppError;
use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
use crate::domain::shared_kernel::services::eps_cache_service::EPSCacheService;
use crate::infrastructure::cache::Cache;
use crate::web::analytics::convert_screening_result_to_eps_ranking;
use super::{
  dto::*,
  enhancement::enhance_with_websocket_data,
  transform::{
    transform_ranking_to_unified_format,
    transform_unified_to_card_format,
  },
  metadata::{ get_available_countries_static, get_available_sectors_static },
};

/// GET /api/v1/analytics/rankings - Direct TradingView card dashboard endpoint with caching
/// Same API contract as before, now using direct TradingView API calls (bypasses broken DDD adapter)
pub async fn get_unified_analytics_rankings_cached(
  Query(params): Query<EPSRankingQueryParams>,
  Extension(cache): Extension<Arc<dyn Cache>>
) -> Result<Json<CardDashboardResponse>, AppError> {
  debug!(
    "Direct TradingView analytics rankings API called with params: {:?}",
    params
  );

  // Convert query params to service params with defaults
  let limit = params.limit.unwrap_or(10);
  let page = params.page.unwrap_or(1).max(1); // Ensure page is at least 1
  let skip = (page - 1) * limit; // Convert page to skip internally

  // Generate cache key for this request
  let cache_key = generate_cache_key(&params);
  debug!("Generated cache key: {}", cache_key);

  // FIXED: Check cache first (1-hour TTL) - restored cache functionality
  if let Some(cached_data) = cache.get(&cache_key) {
    if
      let Ok(cached_response) = serde_json::from_str::<CardDashboardResponse>(
        &cached_data
      )
    {
      info!("Cache hit for analytics rankings - returning cached data");
      return Ok(Json(cached_response));
    }
  }

  debug!("Cache miss for analytics rankings - fetching fresh data");

  // Log request details for debugging
  info!(
    "Processing direct TradingView analytics rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}",
    params.country,
    params.sort_by,
    page,
    limit
  );

  // Fetch data using direct TradingView API calls (bypasses broken DDD adapter)
  let start_time = std::time::Instant::now();

  // Create TradingView service for REAL API calls using restored modular architecture
  let tradingview_service =
    crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(
      Arc::new(get_fallback_config())
    );

  // Get rankings data directly from TradingView (bypasses broken DDD adapter)
  let (screening_results, total_count) = tradingview_service
    .fetch_eps_growth_ranking(
      Some(skip),
      Some(limit),
      params.country.clone(),
      params.sector.clone(),
      params.sort_by.clone().or(Some("qoq_growth".to_string()))
    ).await
    .map_err(|e|
      AppError::new(
        crate::core::errors::ErrorKind::ExternalServiceError,
        format!("TradingView API error: {}", e)
      )
    )?;

  // Convert TradingView screening results to EPS rankings format while preserving quarterly data
  let rankings_with_quarterly: Vec<(EPSRanking, crate::domain::shared_kernel::entities::market_data::StockScreeningResult)> = screening_results
    .into_iter()
    .map(|result| {
      let ranking = convert_screening_result_to_eps_ranking(result.clone());
      (ranking, result)
    })
    .collect();
  
  // Extract just rankings for the existing logic
  let mut rankings_data: Vec<EPSRanking> = rankings_with_quarterly
    .iter()
    .map(|(ranking, _)| ranking.clone())
    .collect();

  // ENABLED: WebSocket enhancement with performance optimizations
  // Limit to small batches and add timeout protection
  if rankings_data.len() <= 10 && !rankings_data.is_empty() {
    debug!(
      "Direct endpoint: Enhancing {} rankings with WebSocket EPS data (performance optimized)",
      rankings_data.len()
    );

    let symbols: Vec<String> = rankings_data
      .iter()
      .map(|r| r.symbol.clone())
      .collect();

    // Add timeout protection for WebSocket enhancement
    let enhancement_timeout = tokio::time::Duration::from_secs(5);
    let enhancement_result = tokio::time::timeout(
      enhancement_timeout,
      enhance_with_websocket_data(&symbols, &mut rankings_data)
    ).await;

    match enhancement_result {
      Ok(Ok(enhanced_count)) => {
        info!("Direct endpoint: Enhanced {} rankings with WebSocket data in <5s", enhanced_count);
      }
      Ok(Err(e)) => {
        warn!("Direct endpoint: WebSocket enhancement failed: {}, using Scanner API data", e);
      }
      Err(_) => {
        warn!("Direct endpoint: WebSocket enhancement timed out after 5s, using Scanner API data");
      }
    }
  } else if rankings_data.len() > 10 {
    debug!("Direct endpoint: Skipping WebSocket enhancement for {} items (performance limit)", rankings_data.len());
  }
  
  info!("Using TradingView Scanner API data with optional WebSocket real-time enhancement");

  // Transform EPS rankings to unified format first, then to card format with quarterly data
  let unified_rankings: Vec<super::dto::UnifiedRankingItem> = rankings_data
    .into_iter()
    .enumerate()
    .map(|(index, ranking)| {
      transform_ranking_to_unified_format(ranking, index + (skip as usize) + 1)
    })
    .collect();

  // Transform to card format for the response, including quarterly EPS data
  let card_data: Vec<SymbolCardData> = unified_rankings
    .into_iter()
    .enumerate()
    .map(|(index, unified_ranking)| {
      let mut card_data = transform_unified_to_card_format(unified_ranking);
      
      // Add quarterly EPS data from the original screening result if available
      if let Some((_, screening_result)) = rankings_with_quarterly.get(index) {
        card_data.eps_quarterly = super::transform::transform_stock_screening_to_quarterly_data(screening_result);
      }
      
      card_data
    })
    .collect();

  // Calculate pagination metadata
  let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;
  let has_next = page < total_pages;
  let has_prev = page > 1;

  // Prepare metadata - using direct TradingView API
  let metadata = CardDashboardMetadata {
    available_countries: get_available_countries_static(),
    available_sectors: get_available_sectors_static(),
    request_timestamp: chrono::Utc::now(),
    data_source: "live_tradingview_api".to_string(),
  };

  let duration = start_time.elapsed();

  // DEBUG: Capture final DTO structure before JSON serialization
  let _dto_debug = card_data
    .iter()
    .take(3)
    .map(|card| {
      let quarters_debug = card.quarterly_performance
        .iter()
        .take(2)
        .map(|q| {
          format!(
            "  Quarter: '{}', Date: '{}', EPS: {:.2}, Price: {:.2}",
            q.quarter,
            q.date,
            q.eps,
            q.price
          )
        })
        .collect::<Vec<_>>()
        .join("\n");
      format!(
        "Symbol: {}, Rank: {}, Status: '{}', Value: {:.2}\nQuarterly Performance:\n{}",
        card.symbol,
        card.rank,
        card.active_status,
        card.value,
        quarters_debug
      )
    })
    .collect::<Vec<_>>()
    .join("\n\n");

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
    message: Some(
      format!("Fetched {} card dashboard rankings successfully from TradingView API", data_len)
    ),
    processing_time_ms: duration.as_millis() as u64,
  };

  info!(
    "Direct TradingView API card dashboard completed in {:?} - {} items returned",
    duration,
    data_len
  );

  // Store response in cache with 1-hour TTL (3600 seconds)
  if let Ok(serialized_response) = serde_json::to_string(&card_response) {
    cache.set(&cache_key, serialized_response, Some(3600));
    debug!("Successfully cached analytics rankings with key: {}", cache_key);
  }

  Ok(Json(card_response))
}

/// GET /api/v1/analytics/cache/stats - Get cache statistics
pub async fn get_cache_stats(Extension(
  cache_service,
): Extension<Arc<EPSCacheService>>) -> Result<
  Json<CacheStatsResponse>,
  AppError
> {
  debug!("Getting cache statistics");

  let stats = cache_service.get_cache_stats().await;

  let response = CacheStatsResponse {
    success: true,
    stats,
    message: "Cache statistics retrieved successfully".to_string(),
    timestamp: chrono::Utc::now(),
  };

  info!(
    "Cache stats - Total entries: {}, Active: {}, Hit ratio: {:.2}%",
    response.stats.total_entries,
    response.stats.active_entries,
    response.stats.hit_ratio * 100.0
  );

  Ok(Json(response))
}

/// POST /api/v1/analytics/cache/refresh - Force cache refresh
pub async fn force_cache_refresh(Extension(
  cache_service,
): Extension<Arc<EPSCacheService>>) -> Result<
  Json<CacheRefreshResponse>,
  AppError
> {
  info!("Forcing cache refresh");

  let start_time = std::time::Instant::now();
  let refreshed_count = cache_service
    .refresh_cache().await
    .map_err(|e|
      AppError::new(
        crate::core::errors::ErrorKind::ExternalServiceError,
        e.to_string()
      )
    )?;
  let duration = start_time.elapsed();

  let response = CacheRefreshResponse {
    success: true,
    refreshed_entries: refreshed_count as usize,
    duration_ms: duration.as_millis() as u64,
    message: format!("Cache refreshed with {} entries", refreshed_count),
    timestamp: chrono::Utc::now(),
  };

  info!(
    "Cache refresh completed - {} entries refreshed in {:?}",
    refreshed_count,
    duration
  );

  Ok(Json(response))
}

/// GET /api/v1/analytics/cache/health - Cache health check
pub async fn cache_health_check(Extension(
  cache_service,
): Extension<Arc<EPSCacheService>>) -> Result<
  Json<CacheHealthResponse>,
  AppError
> {
  debug!("Checking cache health");

  let stats = cache_service.get_cache_stats().await;

  // Determine health status based on cache metrics
  let healthy = stats.active_entries > 0 && stats.hit_ratio > 0.1;
  let status = if healthy { "healthy" } else { "degraded" };

  let mut recommendations = Vec::new();

  if stats.active_entries == 0 {
    recommendations.push(
      "Cache is empty - consider warming the cache".to_string()
    );
  }

  if stats.hit_ratio < 0.5 {
    recommendations.push(
      "Low cache hit ratio - consider increasing TTL".to_string()
    );
  }

  if stats.cache_size_mb > 100.0 {
    recommendations.push(
      "Cache size is large - monitor memory usage".to_string()
    );
  }

  let response = CacheHealthResponse {
    status: status.to_string(),
    healthy,
    cache_stats: stats,
    recommendations,
    timestamp: chrono::Utc::now(),
  };

  info!(
    "Cache health check - Status: {}, Active entries: {}, Hit ratio: {:.2}%",
    status,
    response.cache_stats.active_entries,
    response.cache_stats.hit_ratio * 100.0
  );

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
  crate::config::get_fallback_config()
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
    assert_eq!(config.backend_url, "http://localhost:8080");
    assert!(!config.firebase_project_id.is_empty());
  }
}
