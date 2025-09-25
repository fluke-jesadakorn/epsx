pub mod eps_handlers;
pub mod eps;  // New focused modules architecture

use axum::{
    routing::{get, post},
    Router,
    Extension,
    extract::{Request, Path},
    http::StatusCode,
    response::{Json, Response, IntoResponse},
    middleware::{from_fn, Next},
};
use serde::Deserialize;

use crate::infrastructure::adapters::services::tradingview::TradingViewApiService;
// use crate::infrastructure::container::InfraFactory; // Removed - no longer exists
use crate::config::Config;
use crate::infrastructure::cache::{CacheFactory, Cache};
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

    // Create unified cache service (automatically selects InMemory or Redis)
    let cache_box = CacheFactory::with_fallback().await;
    let unified_cache_service: std::sync::Arc<dyn Cache> = std::sync::Arc::from(cache_box);

    // Background cache refresh removed - using on-demand loading instead
    
    // Create DDD Stock Analysis Repository Adapter
    // For now, create a mock EPS repository since this is bridging legacy systems
    // struct MockEPSRepository;
    // Legacy MockEPSRepository implementation removed - use actual repository implementations
    // 
    // Create stock analysis adapter for the repository layer
    let eps_repository = std::sync::Arc::new(crate::infrastructure::adapters::repositories::EPSRepositoryAdapter::new());
    let eps_service = std::sync::Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository));
    let _stock_analysis_adapter = std::sync::Arc::new(
        crate::infrastructure::adapters::repositories::StockAnalysisRepositoryAdapter::new(
            eps_service
        )
    );

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
        // TODO: Axum 0.7.9 trait bound issue - use stateless_auth_middleware temporarily
        .layer(from_fn(crate::web::middleware::stateless_auth_middleware))
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
        // TODO: Axum 0.7.9 trait bound issue - use stateless_auth_middleware temporarily  
        .layer(from_fn(crate::web::middleware::stateless_auth_middleware))
        .layer(from_fn(require_analytics_permission));

    // Public routes (no authentication required) - use simple test handler first
    let public_routes = Router::new()
        .route("/api/v1/public/analytics/rankings", get(simple_rankings_handler))
        .route("/api/v1/public/analytics/eps-rankings", get(simple_rankings_handler))
        .route("/api/v1/public/analytics/filters", get(eps_handlers::get_filter_options))
        .route("/api/v1/public/analytics/countries", get(eps_handlers::get_available_countries))
        .route("/api/v1/public/analytics/sectors", get(eps_handlers::get_sectors_by_country));
        // No authentication middleware for public routes - test with simple handler first

    let eps_repository_clone = std::sync::Arc::new(crate::infrastructure::adapters::repositories::EPSRepositoryAdapter::new());
    let eps_service_clone = std::sync::Arc::new(crate::domain::shared_kernel::services::eps_ranking_service::EPSRankingService::new(eps_repository_clone));

    Router::new()
        .merge(v1_routes)
        .merge(legacy_routes)
        .merge(public_routes)
        // Add EPS service extension (cache extensions already added to individual route groups)
        .layer(Extension(eps_service_clone))
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

/// Simple test handler for public analytics rankings (no extensions required)
async fn simple_rankings_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    let mock_response = serde_json::json!({
        "success": true,
        "data": [
            {
                "rank": 1,
                "symbol": "AAPL",
                "name": "Apple Inc.",
                "eps_growth": 15.5,
                "market_cap": 3000000000000_u64,
                "sector": "Technology"
            },
            {
                "rank": 2,
                "symbol": "MSFT",
                "name": "Microsoft Corp.",
                "eps_growth": 12.3,
                "market_cap": 2800000000000_u64,
                "sector": "Technology"
            }
        ],
        "pagination": {
            "page": 1,
            "limit": 5,
            "total": 2,
            "total_pages": 1
        },
        "message": "Mock analytics data for testing"
    });
    
    Ok(Json(mock_response))
}
