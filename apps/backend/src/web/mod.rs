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
pub mod admin_assignment;
// ⚡ CRITICAL: Comprehensive Error System (Phase 1.3)
pub mod errors;
pub mod public;

use axum::{ routing::get, Router, http::Method };
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::middleware as axum_middleware;

use crate::infrastructure::container::DomainContainer;

// Legacy handlers removed - replaced by unified health module and analytics handlers
// health_handler -> health::health_check_handler  
// cache_handler -> analytics::eps_handlers::force_cache_refresh
// premium_rankings_handler -> analytics::eps_handlers::get_unified_analytics_rankings_cached


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
      // Pure Web3 authentication headers (CRITICAL FOR WALLET AUTH)
      HeaderName::from_static("x-wallet-address"),
      HeaderName::from_static("x-chain-id"),
      HeaderName::from_static("x-signature"),
      HeaderName::from_static("x-message"),
      HeaderName::from_static("x-timestamp"),
      HeaderName::from_static("x-nonce"),
    ])
    .expose_headers([
      HeaderName::from_static("x-request-id"),
      HeaderName::from_static("x-rate-limit-remaining"),
      HeaderName::from_static("x-rate-limit-reset"),
    ])
    .allow_credentials(false) // Must be false when using Any origin
    .max_age(Duration::from_secs(86400)) // 24 hours
}

// create_standalone_analytics_routes function removed
// Analytics routes are now handled by UnifiedRouteBuilder

/// Create the main application router with simplified, clean architecture
/// Eliminates all route duplication and over-engineering with single source of truth
pub fn create_router(container: Arc<DomainContainer>) -> Router {
  // Use simple route builder - single source of truth
  // This returns Router<()> with unified middleware built in
  let simple_router = routes::SimpleRouteBuilder::new(container.clone())
    .build();

  // Configure CORS for all routes
  let cors = configure_cors_for_frontend();

  // Apply minimal, essential middleware stack only
  simple_router
    // Essential security headers
    .layer(axum_middleware::from_fn(
      crate::web::middleware::security_headers_middleware
    ))
    // Request ID for tracing
    .layer(axum_middleware::from_fn(
      crate::web::middleware::request_id_middleware
    ))
    // CORS (must be last)
    .layer(cors)
}

/// Create a demo router for Cloud Run demonstration without database dependencies
pub async fn create_demo_router() -> Router {
  use axum::response::Json;
  use serde_json::json;

  Router::new()
    // Use unified health endpoints from health module
    .route("/health", get(health::health_check_handler))
    .route("/readiness", get(health::readiness_check_handler))
    .route("/liveness", get(health::liveness_check_handler))
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
