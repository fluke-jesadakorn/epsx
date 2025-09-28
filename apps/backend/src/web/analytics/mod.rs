pub mod eps_handlers;
pub mod eps;  // New focused modules architecture

use axum::{
    routing::{get, post},
    Router,
    Extension,
    extract::{Request, Path, Query},
    http::StatusCode,
    response::{Json, Response, IntoResponse},
    middleware::{from_fn, Next},
};
use serde::Deserialize;

use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
// use crate::infrastructure::container::InfraFactory; // Removed - no longer exists
use crate::config::Config;
use crate::infrastructure::cache::{Cache, ServerlessCacheFactory};
// AuthenticatedUser moved to domain layer - define locally for now
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub permissions: Vec<String>,
}

pub use eps_handlers::*;

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub granularity: Option<String>,
}


pub async fn create_analytics_router() -> Router {
    // Create services for both database and cache approaches
    
    // Create cache-based EPS service with TradingView integration
    let config = match Config::from_env() {
        Ok(config) => std::sync::Arc::new(config),
        Err(e) => {
            tracing::warn!("Failed to load config, using fallback: {:?}", e);
            // Use a minimal config that should work for basic operation
            std::sync::Arc::new(crate::config::get_fallback_config())
        }
    };
    let _tradingview_service = std::sync::Arc::new(TradingViewApiService::new(config.clone()));

    // Create unified cache service (Redis-only for serverless)
    let unified_cache_service: std::sync::Arc<dyn Cache> = ServerlessCacheFactory::redis_only_arc().await
        .unwrap_or_else(|e| {
            tracing::warn!("Redis cache creation failed: {}, falling back to minimal cache", e);
            std::sync::Arc::new(crate::infrastructure::cache::MemoryCache::new())
        });

    // Background cache refresh removed - using on-demand loading instead
    
    // Create DDD Stock Analysis Repository Adapter
    // For now, create a mock EPS repository since this is bridging legacy systems
    // struct MockEPSRepository;
    // Legacy MockEPSRepository implementation removed - use actual repository implementations
    // 
    // Note: EPS repository implementation removed - using direct TradingView integration

    // Create versioned routes with permission middleware
    let v1_routes = Router::new()
        // Main EPS rankings endpoints (RESTful structure) - now internally using DDD Trading Analytics
        .route("/api/v1/analytics/eps-rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        // Compatibility endpoint - same handler, different path
        .route("/api/v1/analytics/rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        // All existing endpoints continue to work with same API contract
        .route("/api/v1/analytics/eps-rankings/countries", get(eps_handlers::get_available_countries))
        .route("/api/v1/analytics/eps-rankings/countries/all", get(eps_handlers::get_all_valid_countries))
        .route("/api/v1/analytics/eps-rankings/sectors", get(eps_handlers::get_sectors_by_country))
        .route("/api/v1/analytics/eps-rankings/health", get(eps_handlers::eps_health_check))
        // Simplified analytics endpoints for frontend compatibility
        .route("/api/v1/analytics/filters", get(eps_handlers::get_filter_options))
        .route("/api/v1/analytics/countries", get(eps_handlers::get_available_countries))
        .route("/api/v1/analytics/sectors", get(eps_handlers::get_sectors_by_country))
        // Cache management endpoints - require epsx:analytics:manage permission
        .route("/api/v1/analytics/cache/stats", get(eps_handlers::get_cache_stats))
        .route("/api/v1/analytics/cache/refresh", post(eps_handlers::force_cache_refresh))
        .route("/api/v1/analytics/cache/health", get(eps_handlers::cache_health_check))
        // Admin cache management endpoints (namespace consistency)
        .route("/api/v1/admin/cache/stats", get(eps_handlers::get_cache_stats))
        .route("/api/v1/admin/cache/refresh", post(eps_handlers::force_cache_refresh))
        .route("/api/v1/admin/cache/health", get(eps_handlers::cache_health_check))
        // System metrics endpoint for admin dashboard
        .route("/api/v1/admin/analytics/metrics", get(system_metrics_handler))
        // Admin analytics endpoints for dashboard
        .route("/api/v1/admin/analytics/time-series", get(admin_time_series_handler))
        .route("/api/v1/admin/analytics/modules", get(admin_modules_handler))
        // Stock ranking assignment endpoints
        .route("/api/v1/admin/stock-ranking/assignments", get(stock_ranking_assignments_handler))
        .route("/api/v1/admin/stock-ranking/assignments/:assignment_id/extend", post(extend_assignment_handler))
        .route("/api/v1/admin/stock-ranking/assignments/:assignment_id/revoke", post(revoke_assignment_handler))
        // Add cache extension before middleware
        .layer(Extension(unified_cache_service.clone()))
        // No longer needs DDD adapter - using direct TradingView API
        // Apply user authentication middleware
        // TODO: Temporarily disabled due to Axum trait bound issues
        // .layer(from_fn(crate::web::middleware::web3_auth_middleware))
        // Apply analytics view permission requirement to all analytics routes
        .layer(from_fn(require_analytics_permission));

    // Legacy routes for backward compatibility with permission enforcement
    let legacy_routes = Router::new()
        .route("/analytics/rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        .route("/analytics/eps-rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        .route("/analytics/eps-rankings/countries", get(eps_handlers::get_available_countries))
        .route("/analytics/eps-rankings/countries/all", get(eps_handlers::get_all_valid_countries))
        .route("/analytics/eps-rankings/sectors", get(eps_handlers::get_sectors_by_country))
        .route("/analytics/eps-rankings/health", get(eps_handlers::eps_health_check))
        .route("/analytics/system/metrics", get(system_metrics_handler))
        // V1 prefix routes (also legacy - should use /api/v1/ instead)
        .route("/v1/analytics/rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        .route("/v1/analytics/eps-rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        .route("/v1/analytics/eps-rankings/countries", get(eps_handlers::get_available_countries))
        .route("/v1/analytics/eps-rankings/countries/all", get(eps_handlers::get_all_valid_countries))
        .route("/v1/analytics/eps-rankings/sectors", get(eps_handlers::get_sectors_by_country))
        .route("/v1/analytics/eps-rankings/health", get(eps_handlers::eps_health_check))
        .route("/v1/analytics/cache/stats", get(eps_handlers::get_cache_stats))
        .route("/v1/analytics/cache/refresh", post(eps_handlers::force_cache_refresh))
        .route("/v1/analytics/cache/health", get(eps_handlers::cache_health_check))
        // Add cache extension before middleware
        .layer(Extension(unified_cache_service.clone()))
        // Apply same permission middleware to legacy routes
        // TODO: Temporarily disabled due to Axum trait bound issues
        // .layer(from_fn(crate::web::middleware::web3_auth_middleware))
        .layer(from_fn(require_analytics_permission));

    // Public routes (no authentication required) - use simple test handler first
    let public_routes = Router::new()
        .route("/api/v1/public/analytics/rankings", get(simple_rankings_handler))
        .route("/api/v1/public/analytics/eps-rankings", get(simple_rankings_handler))
        .route("/api/v1/public/analytics/filters", get(eps_handlers::get_filter_options))
        .route("/api/v1/public/analytics/countries", get(eps_handlers::get_available_countries))
        .route("/api/v1/public/analytics/sectors", get(eps_handlers::get_sectors_by_country))
        // Portfolio routes - positive growth only
        .route("/api/v1/portfolio/rankings", get(portfolio_rankings_handler))
        .route("/api/v1/public/portfolio/rankings", get(portfolio_rankings_handler));
        // No authentication middleware for public routes - test with simple handler first

    // Note: EPS repository implementation removed - using direct TradingView integration

    Router::new()
        .merge(v1_routes)
        .merge(legacy_routes)
        .merge(public_routes)
        // Add EPS service extension (cache extensions already added to individual route groups)
        // Note: EPS service extension removed with repository cleanup
}

/// System metrics handler for admin dashboard
/// GET /api/v1/analytics/system/metrics
async fn system_metrics_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock implementation - replace with actual system metrics collection
    let metrics = serde_json::json!({
        "totalRequests": 1245678,
        "totalUsers": 8934,
        "totalRevenue": 45672.89,
        "averageResponseTime": 245,
        "errorRate": 0.23,
        "activeApiKeys": 156
    });
    
    Ok(Json(metrics))
}

/// Admin time series data handler for dashboard
async fn admin_time_series_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // Generate mock time series data - replace with actual analytics data
    let mut time_series_data = Vec::new();
    
    for i in 0..7 {
        let date = chrono::Utc::now() - chrono::Duration::days(i);
        time_series_data.push(serde_json::json!({
            "date": date.format("%Y-%m-%d").to_string(),
            "requests": 40000 + (i * 1500) as u32,
            "users": 800 + (i * 50) as u32,
            "revenue": 3000 + (i * 200) as u32,
            "errors": 10 + (i * 2) as u32
        }));
    }
    
    let response = serde_json::json!({
        "data": time_series_data
    });
    
    Ok(Json(response))
}

/// Admin modules data handler for dashboard
async fn admin_modules_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock module usage data - replace with actual module analytics
    let modules = vec![
        serde_json::json!({
            "moduleName": "User Management",
            "requests": 450000,
            "users": 3200,
            "revenue": 15000,
            "quota": 500000,
            "quotaUsed": 450000
        }),
        serde_json::json!({
            "moduleName": "Analytics",
            "requests": 320000,
            "users": 2100,
            "revenue": 12000,
            "quota": 400000,
            "quotaUsed": 320000
        }),
        serde_json::json!({
            "moduleName": "API Gateway",
            "requests": 475678,
            "users": 3634,
            "revenue": 18672.89,
            "quota": 600000,
            "quotaUsed": 475678
        })
    ];
    
    let response = serde_json::json!({
        "modules": modules
    });
    
    Ok(Json(response))
}

/// Stock ranking assignments handler for admin dashboard
async fn stock_ranking_assignments_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock stock ranking assignments data - replace with actual database queries
    let assignments = vec![
        serde_json::json!({
            "id": "assignment-1",
            "userId": "user-123",
            "userEmail": "user@example.com",
            "packageName": "Premium Analytics",
            "assignedAt": "2025-01-01T00:00:00Z",
            "expiresAt": "2025-12-31T23:59:59Z",
            "status": "active"
        }),
        serde_json::json!({
            "id": "assignment-2",
            "userId": "user-456",
            "userEmail": "admin@example.com",
            "packageName": "Basic Rankings",
            "assignedAt": "2025-02-01T00:00:00Z",
            "expiresAt": "2025-08-31T23:59:59Z",
            "status": "active"
        })
    ];
    
    let response = serde_json::json!({
        "assignments": assignments,
        "total": assignments.len(),
        "success": true
    });
    
    Ok(Json(response))
}

/// Extend stock ranking assignment handler
async fn extend_assignment_handler(Path(assignment_id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock implementation - in production, this would update the database
    tracing::info!("Extending assignment: {}", assignment_id);
    
    let response = serde_json::json!({
        "success": true,
        "message": format!("Assignment {} extended successfully", assignment_id),
        "assignmentId": assignment_id,
        "newExpiryDate": (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339()
    });
    
    Ok(Json(response))
}

/// Revoke stock ranking assignment handler
async fn revoke_assignment_handler(Path(assignment_id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock implementation - in production, this would update the database
    tracing::info!("Revoking assignment: {}", assignment_id);
    
    let response = serde_json::json!({
        "success": true,
        "message": format!("Assignment {} revoked successfully", assignment_id),
        "assignmentId": assignment_id,
        "revokedAt": chrono::Utc::now().to_rfc3339()
    });
    
    Ok(Json(response))
}

/// Analytics permission middleware - requires epsx:analytics:view permission
async fn require_analytics_permission(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get authenticated user from request extensions (should be set by user_auth_middleware)
    let user = request.extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| create_analytics_unauthorized_response("User not authenticated"))?;

    // Check if user has required permission (supports wildcards and embedded timestamps)
    let required_permission = "epsx:analytics:view";
    if !user.permissions.iter().any(|p| permission_matches(p, required_permission)) {
        tracing::info!(
            "User {} lacks required permission '{}' for analytics endpoint {}",
            user.id,
            required_permission,
            request.uri().path()
        );
        return Err(create_analytics_forbidden_response(&format!(
            "Permission '{}' required for analytics access", 
            required_permission
        )));
    }

    tracing::debug!(
        "User {} granted analytics access with permission check passed",
        user.id
    );

    Ok(next.run(request).await)
}

/// Check if user permission matches required permission (supports wildcards and embedded timestamps)
fn permission_matches(user_permission: &str, required_permission: &str) -> bool {
    // First check for embedded timestamp permissions (format: platform:resource:action:timestamp)
    let parts: Vec<&str> = user_permission.split(':').collect();
    if parts.len() == 4 {
        // This is an embedded timestamp permission - check if it's expired
        if let Ok(expiry_timestamp) = parts[3].parse::<i64>() {
            let current_timestamp = chrono::Utc::now().timestamp();
            if current_timestamp > expiry_timestamp {
                // Permission has expired
                tracing::debug!("Embedded permission {} has expired (current: {}, expiry: {})", 
                    user_permission, current_timestamp, expiry_timestamp);
                return false;
            }
            // Check the base permission (without timestamp)
            let base_permission = parts[0..3].join(":");
            return permission_matches(&base_permission, required_permission);
        }
    }
    
    // Exact match (fastest)
    if user_permission == required_permission {
        return true;
    }
    
    // Wildcard matching
    if user_permission.ends_with(":*:*") {
        let prefix = &user_permission[..user_permission.len() - 4];
        return required_permission.starts_with(prefix);
    }
    
    if user_permission.ends_with(":*") {
        let prefix = &user_permission[..user_permission.len() - 2];
        return required_permission.starts_with(prefix);
    }
    
    false
}

/// Create analytics unauthorized response
fn create_analytics_unauthorized_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "unauthorized",
        "message": message,
        "context": "analytics",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::UNAUTHORIZED,
        [("Content-Type", "application/json")],
        error_body.to_string()
    ).into_response()
}

/// Create analytics forbidden response
fn create_analytics_forbidden_response(message: &str) -> Response {
    let error_body = serde_json::json!({
        "error": "forbidden",
        "message": message,
        "context": "analytics",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    (
        StatusCode::FORBIDDEN,
        [("Content-Type", "application/json")],
        error_body.to_string()
    ).into_response()
}

/// Real TradingView handler for public analytics rankings
async fn simple_rankings_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(10);
    let country = params.get("country").cloned();
    let sector = params.get("sector").cloned();
    let sort_by = params.get("sort_by").cloned();
    
    tracing::info!("🌐 Public TradingView API request - Page: {}, Limit: {}, Country: {:?}, Sector: {:?}", 
                  page, limit, country, sector);
    
    // Create TradingView service for REAL API calls
    let tradingview_service = std::sync::Arc::new(
        crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(
            std::sync::Arc::new(crate::config::get_fallback_config())
        )
    );
    
    // Calculate skip for TradingView API
    let skip = (page - 1) * limit;
    
    // Get real data from TradingView Scanner API
    let (screening_results, total_count) = match tradingview_service
        .fetch_eps_growth_ranking(
            Some(skip),
            Some(limit),
            country,
            sector,
            sort_by.or(Some("qoq_growth".to_string()))
        ).await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("❌ TradingView API error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    tracing::info!("✅ Retrieved {} real stocks from TradingView API, total available: {}", 
                  screening_results.len(), total_count);
    
    // Convert screening results to frontend format
    let rankings: Vec<serde_json::Value> = screening_results
        .into_iter()
        .enumerate()
        .map(|(i, result)| {
            let rank = (skip as usize) + i + 1;
            let eps_growth = result.eps_growth_yoy.unwrap_or(result.change_percent);
            let current_eps = result.current_eps.unwrap_or_else(|| {
                // Calculate EPS from price/PE if not available
                if let Some(pe_ratio) = result.pe_ratio {
                    result.price / pe_ratio.max(1.0)
                } else {
                    0.0
                }
            });
            
            serde_json::json!({
                "rank": rank,
                "symbol": result.symbol,
                "name": result.name,
                "latest_date": "2024-Q4",
                "value": result.price,
                "active_status": if eps_growth > 0.0 { "TRACK" } else { "STOP" },
                "quarterly_performance": [
                    {
                        "quarter": "Q4 2024",
                        "date": "Dec 31, 2024",
                        "price": result.price,
                        "eps": current_eps,
                        "eps_growth": eps_growth,
                        "price_growth": result.change_percent,
                        "is_estimated": false
                    }
                ],
                "next_quarter_estimate": {
                    "quarter": "2025-Q1",
                    "estimated_eps": current_eps * 1.05, // 5% growth estimate
                    "announcement_date": "Est. Feb 15, 2025",
                    "announcement_timestamp": chrono::Utc::now().timestamp() + (45 * 24 * 60 * 60),
                    "days_until_announcement": 45,
                    "confidence": if eps_growth > 10.0 { "High" } else if eps_growth > 0.0 { "Medium" } else { "Low" }
                },
                "eps_growth": eps_growth,
                "market_cap": result.market_cap.unwrap_or(0.0) as u64,
                "sector": result.sector.unwrap_or("Unknown".to_string()),
                "currency": result.currency.unwrap_or("USD".to_string())
            })
        })
        .collect();
    
    // Calculate pagination metadata
    let total_pages = ((total_count as f64) / (limit as f64)).ceil() as i32;
    let has_next = page < total_pages;
    let has_prev = page > 1;
    
    let real_response = serde_json::json!({
        "success": true,
        "rankings": rankings,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total_count,
            "totalPages": total_pages,
            "hasNext": has_next,
            "hasPrev": has_prev
        },
        "message": "Real TradingView data via Scanner API"
    });
    
    tracing::info!("📊 Returning {} real rankings from TradingView, page {} of {}", 
                  rankings.len(), page, total_pages);
    
    Ok(Json(real_response))
}

/// Portfolio rankings handler with positive growth filtering
async fn portfolio_rankings_handler(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = params.get("page").and_then(|p| p.parse::<i32>().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse::<i32>().ok()).unwrap_or(10);
    let country = params.get("country").cloned();
    let sector = params.get("sector").cloned();
    let sort_by = params.get("sort_by").cloned();
    
    tracing::info!("💼 Portfolio TradingView API request - Page: {}, Limit: {}, Country: {:?}, Sector: {:?}", 
                  page, limit, country, sector);
    
    // Create TradingView service for REAL API calls
    let tradingview_service = std::sync::Arc::new(
        crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(
            std::sync::Arc::new(crate::config::get_fallback_config())
        )
    );
    
    // Fetch more data to account for filtering out negative growth
    let fetch_limit = limit * 3; // Get 3x more to ensure we have enough positive results
    let skip = (page - 1) * limit;
    let fetch_skip = (page - 1) * fetch_limit;
    
    // Get real data from TradingView Scanner API
    let (screening_results, _total_count) = match tradingview_service
        .fetch_eps_growth_ranking(
            Some(fetch_skip),
            Some(fetch_limit),
            country,
            sector,
            sort_by.or(Some("qoq_growth".to_string()))
        ).await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("❌ TradingView API error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Filter for positive growth only
    let positive_results: Vec<_> = screening_results
        .into_iter()
        .filter(|result| {
            let growth = result.eps_growth_yoy.unwrap_or(result.change_percent);
            growth > 0.0 // Only positive growth
        })
        .skip(skip as usize)
        .take(limit as usize)
        .collect();
    
    tracing::info!("✅ Filtered to {} positive growth stocks from TradingView API", 
                  positive_results.len());
    
    // Convert screening results to frontend format
    let rankings: Vec<serde_json::Value> = positive_results
        .into_iter()
        .enumerate()
        .map(|(i, result)| {
            let rank = (skip as usize) + i + 1;
            let eps_growth = result.eps_growth_yoy.unwrap_or(result.change_percent);
            let current_eps = result.current_eps.unwrap_or_else(|| {
                // Calculate EPS from price/PE if not available
                if let Some(pe_ratio) = result.pe_ratio {
                    result.price / pe_ratio.max(1.0)
                } else {
                    0.0
                }
            });
            
            serde_json::json!({
                "rank": rank,
                "symbol": result.symbol,
                "name": result.name,
                "latest_date": "2024-Q4",
                "value": result.price,
                "active_status": "TRACK", // All positive growth stocks are TRACK
                "quarterly_performance": [
                    {
                        "quarter": "Q4 2024",
                        "date": "Dec 31, 2024",
                        "price": result.price,
                        "eps": current_eps,
                        "eps_growth": eps_growth,
                        "price_growth": result.change_percent,
                        "is_estimated": false
                    }
                ],
                "next_quarter_estimate": {
                    "quarter": "2025-Q1",
                    "estimated_eps": current_eps * 1.05, // 5% growth estimate
                    "announcement_date": "Est. Feb 15, 2025",
                    "announcement_timestamp": chrono::Utc::now().timestamp() + (45 * 24 * 60 * 60),
                    "days_until_announcement": 45,
                    "confidence": if eps_growth > 10.0 { "High" } else { "Medium" }
                },
                "eps_growth": eps_growth,
                "market_cap": result.market_cap.unwrap_or(0.0) as u64,
                "sector": result.sector.unwrap_or("Unknown".to_string()),
                "currency": result.currency.unwrap_or("USD".to_string())
            })
        })
        .collect();
    
    // Portfolio response - simplified pagination since we're filtering
    let portfolio_response = serde_json::json!({
        "success": true,
        "rankings": rankings,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": rankings.len() * 2, // Estimate for positive results
            "totalPages": ((rankings.len() * 2) as f64 / limit as f64).ceil() as i32,
            "hasNext": rankings.len() == limit as usize,
            "hasPrev": page > 1
        },
        "message": "Portfolio data - positive growth only"
    });
    
    tracing::info!("💼 Returning {} positive portfolio rankings, page {}", 
                  rankings.len(), page);
    
    Ok(Json(portfolio_response))
}
