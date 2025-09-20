// Web layer implementation

pub mod api;
pub mod auth;
pub mod admin;
pub mod routes; // New contextual route architecture
// Removed: permission_profile, permissions - replaced by auth/roles.rs
pub mod user;
pub mod middleware;
pub mod modules;
pub mod validation;
pub mod health;
pub mod analytics;
pub mod settings;
pub mod templates;
pub mod admin_assignment;
pub mod session_management_handlers;
pub mod session_management_routes;
pub mod notifications;

use axum::{ routing::{ get, post }, Router, response::Json, http::Method };
use serde_json::{ json, Value };
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::middleware as axum_middleware;

use crate::infrastructure::container::AppContainer;

/// Health check handler
pub async fn health_handler() -> Json<Value> {
  Json(
    json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend"
    })
  )
}

/// Cache handler (placeholder)
pub async fn cache_handler() -> Json<Value> {
  Json(
    json!({
        "status": "cache_cleared",
        "timestamp": chrono::Utc::now()
    })
  )
}

/// Premium rankings handler (placeholder)
pub async fn premium_rankings_handler() -> Json<Value> {
  Json(
    json!({
        "rankings": [],
        "last_updated": chrono::Utc::now()
    })
  )
}


/// Configure CORS for frontend applications - Allow any origin with Next.js headers
fn configure_cors_for_frontend() -> CorsLayer {
  use tower_http::cors::Any;
  use axum::http::HeaderName;
  use std::time::Duration;

  // Allow any origin for all environments (per user request)
  // Note: When using Any origin, credentials cannot be allowed per CORS spec
  CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([
      Method::GET,
      Method::POST,
      Method::PUT,
      Method::PATCH,
      Method::DELETE,
      Method::OPTIONS,
    ])
    .allow_headers([
      // Standard HTTP headers
      HeaderName::from_static("accept"),
      HeaderName::from_static("authorization"),
      HeaderName::from_static("content-type"),
      HeaderName::from_static("origin"),
      HeaderName::from_static("referer"),
      // Custom API headers
      HeaderName::from_static("x-api-version"),
      HeaderName::from_static("x-request-id"),
      HeaderName::from_static("x-client-version"),
      HeaderName::from_static("x-admin-session"),
      // Next.js React Server Components header
      HeaderName::from_static("rsc"),
      // Next.js Router headers for prefetching (CRITICAL FOR FRONTEND)
      HeaderName::from_static("next-router-prefetch"),
      HeaderName::from_static("next-router-state-tree"),
      HeaderName::from_static("next-url"),
      HeaderName::from_static("purpose"),
      HeaderName::from_static("x-middleware-prefetch"),
      HeaderName::from_static("x-nextjs-data"),
    ])
    .expose_headers([
      HeaderName::from_static("x-request-id"),
      HeaderName::from_static("x-rate-limit-remaining"),
      HeaderName::from_static("x-rate-limit-reset"),
    ])
    .allow_credentials(false) // Must be false when using Any origin
    .max_age(Duration::from_secs(86400)) // 24 hours
}

/// Create standalone analytics routes without AppState dependency
async fn create_standalone_analytics_routes(
  _infra_factory: &crate::infrastructure::InfraFactory
) -> Router {
  use axum::Extension;

  // Create services for analytics

  // Create cache-based EPS service with TradingView integration
  let config = match crate::config::Config::from_env() {
    Ok(config) => std::sync::Arc::new(config),
    Err(e) => {
      tracing::warn!("Failed to load config, using fallback: {:?}", e);
      // Use a minimal simplified config that should work for basic operation
      std::sync::Arc::new(crate::config::Config {
        database_url: "postgresql://localhost/epsx".to_string(),
        backend_url: "http://localhost:8080".to_string(),
        frontend_url: "http://localhost:3000".to_string(),
        admin_frontend_url: "http://localhost:3001".to_string(),
        jwt_secret: "default-jwt-secret".to_string(),
        oidc_client_id: "epsx-frontend".to_string(),
        oidc_client_secret: "default-secret".to_string(),
        oidc_admin_client_id: "epsx-admin".to_string(),
        oidc_admin_client_secret: "default-secret".to_string(),
        firebase_project_id: "epsx-dev".to_string(),
        firebase_private_key: "-----BEGIN PRIVATE KEY-----\ndefault\n-----END PRIVATE KEY-----".to_string(),
        firebase_client_email: "firebase-adminsdk@epsx-dev.iam.gserviceaccount.com".to_string(),
        ethereum_rpc_url: "https://eth.llamarpc.com".to_string(),
        polygon_rpc_url: "https://polygon.llamarpc.com".to_string(),
        arbitrum_rpc_url: "https://arbitrum.llamarpc.com".to_string(),
        optimism_rpc_url: "https://optimism.llamarpc.com".to_string(),
        base_rpc_url: "https://base.llamarpc.com".to_string(),
        bsc_rpc_url: "https://bsc-dataseed.binance.org".to_string(),
        redis_url: None,
        log_level: "info".to_string(),
      })
    }
  };
  let _tradingview_service = std::sync::Arc::new(
    crate::infrastructure::adapters::services::tradingview::TradingViewApiService::new(
      config.clone()
    )
  );

  // Create unified cache service (automatically selects InMemory or Redis)
  let cache_box = crate::infrastructure::cache::CacheFactory::with_fallback().await;
  let unified_cache_service: std::sync::Arc<dyn crate::infrastructure::cache::Cache> = 
    std::sync::Arc::from(cache_box);

  // Background cache refresh removed - using on-demand loading instead

  Router::new()
    // Primary analytics ranking endpoints (using corrected FQ field logic)
    .route(
      "/api/v1/analytics/rankings",
      get(analytics::eps_handlers::get_unified_analytics_rankings_cached)
    )
    .route(
      "/api/v1/analytics/eps-rankings",
      get(analytics::eps_handlers::get_unified_analytics_rankings_cached)
    )
    .route(
      "/api/v1/analytics/eps-rankings/countries",
      get(analytics::eps_handlers::get_available_countries)
    )
    .route(
      "/api/v1/analytics/eps-rankings/countries/all",
      get(analytics::eps_handlers::get_all_valid_countries)
    )
    .route(
      "/api/v1/analytics/eps-rankings/sectors",
      get(analytics::eps_handlers::get_sectors_by_country)
    )
    .route(
      "/api/v1/analytics/eps-rankings/health",
      get(analytics::eps_handlers::eps_health_check)
    )
    .route(
      "/api/v1/analytics/eps-rankings/sync",
      post(analytics::eps_handlers::trigger_eps_sync)
    )
    .route(
      "/api/v1/analytics/eps-rankings/websocket-debug",
      post(analytics::eps_handlers::debug_websocket_eps)
    )
    .route(
      "/api/v1/analytics/eps-rankings/debug-eps-correction",
      post(analytics::eps_handlers::debug_eps_correction)
    )
    .route(
      "/api/v1/analytics/eps-rankings/debug-ranking-data",
      post(analytics::eps_handlers::debug_ranking_data)
    )
    // Add cache endpoints
    .route(
      "/api/v1/analytics/cache/stats",
      get(analytics::eps_handlers::get_cache_stats)
    )
    .route(
      "/api/v1/analytics/cache/refresh",
      post(analytics::eps_handlers::force_cache_refresh)
    )
    .route(
      "/api/v1/analytics/cache/health",
      get(analytics::eps_handlers::cache_health_check)
    )
    // Add services as extensions
    .layer(Extension(unified_cache_service))
}

/// Create the main application router with analytics support
pub async fn create_router(container: Arc<AppContainer>) -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
  // Create config for email service
  let _config = match crate::config::Config::from_env() {
    Ok(config) => std::sync::Arc::new(config),
    Err(_) => {
      // Use minimal config for DDD migration
      std::sync::Arc::new(crate::config::get_fallback_config())
    }
  };

  // Web3-only authentication - create AppState for admin routes compatibility
  let app_state = container.create_app_state().await
    .map_err(|e| {
      tracing::error!("Failed to create app state: {}", e);
      e
    })?;
  
  // Create new contextual route architecture
  let internal_routes = routes::ContextualRouterBuilder::new(
    routes::AccessContext::Internal,
    container.clone()
  ).build().await?;

  let external_routes = routes::ContextualRouterBuilder::new(
    routes::AccessContext::External,
    container.clone()
  ).build().await?;

  let admin_routes_new = routes::ContextualRouterBuilder::new(
    routes::AccessContext::Admin,
    container.clone()
  ).build().await?;

  // Create admin routes with AppState compatibility  
  let admin_routes = admin::routes::create_admin_routes().with_state(app_state.clone());
  let admin_public_routes = admin::routes::create_admin_public_routes().with_state(app_state.clone());

  // Create stateless notification routes
  let notification_routes = notifications::stateless_handlers::create_routes().with_state((*container).clone());

  // Create analytics routes with permission middleware
  let analytics_routes = analytics::create_analytics_router(&container.infra).await;
  
  // Create marketing API routes (plans, promotions, affiliates)
  let marketing_routes = api::v1::create_plans_router(container.db_pool());
  
  // Create payments API routes
  let payments_routes = api::v1::create_payments_router(container.clone());

  // Create core routes
  let core_routes = Router::new()
    .route("/health", get(health_handler))
    .route("/cache", get(cache_handler))
    .with_state(container.clone());

  // Configure CORS for all routes
  let cors = configure_cors_for_frontend();

  // Create Web3 authentication routes
  let web3_routes = auth::web3_routes::create_routes().with_state((*container).clone());

  // Merge routes with Web3-only authentication
  Ok(core_routes
    .merge(analytics_routes)
    .nest("/api/auth/web3", web3_routes)
    .nest("/api/notifications", notification_routes)
    // New contextual routes with proper prefixes
    .nest("/web", internal_routes)
    .nest("/api/external", external_routes) 
    .nest("/admin", admin_routes_new)
    // Legacy routes for backward compatibility
    .nest("/api/v1/plans", marketing_routes)
    .nest("/api/v1/payments", payments_routes)
    .nest("/api/v1/admin", admin_routes)
    .merge(admin_public_routes)
    // Add comprehensive security middleware stack
    // TODO: Fix middleware state type compatibility
    // .layer(axum_middleware::from_fn_with_state(
    //   container.clone(),
    //   crate::web::middleware::security_event_logging_middleware
    // ))
    // .layer(axum_middleware::from_fn_with_state(
    //   container.clone(),
    //   crate::web::middleware::unified_rate_limit_middleware
    // ))
    // .layer(axum_middleware::from_fn_with_state(
    //   container.clone(),
    //   crate::web::middleware::enhanced_security_monitoring_middleware
    // ))
    // .layer(axum_middleware::from_fn_with_state(
    //   container.clone(),
    //   crate::web::middleware::csp_middleware
    // ))
    .layer(axum_middleware::from_fn(
      crate::web::middleware::security_headers_middleware
    ))
    .layer(axum_middleware::from_fn(
      crate::web::middleware::request_id_middleware
    ))
    .layer(axum_middleware::from_fn(
      crate::web::middleware::performance_headers_middleware
    ))
    // Removed enhanced_cors_middleware as it conflicts with proper CORS layer
    .layer(cors))
}

/// Create a demo router for Cloud Run demonstration without database dependencies
pub async fn create_demo_router() -> Router {
  use axum::response::Json;
  use serde_json::json;

  Router::new()
    .route("/health", get(health_handler))
    .route(
      "/",
      get(|| async {
        Json(
          json!({
            "status": "running",
            "service": "epsx-backend",
            "mode": "demo",
            "message": "Backend is running in demo mode. Database connection required for full functionality.",
            "timestamp": chrono::Utc::now()
        })
        )
      })
    )
}
