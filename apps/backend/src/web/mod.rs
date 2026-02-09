// Web layer implementation

pub mod auth;
pub mod admin;
pub mod routes;
pub mod user;
pub mod middleware;
pub mod validation;
pub mod health;
pub mod analytics;
pub mod admin_assignment;
pub mod notifications;
pub mod payments;
// ⚡ CRITICAL: Comprehensive Error System (Phase 1.3)
pub mod errors;
pub mod responses; // Unified API response format
pub mod api_response; // New Standard Response Format
pub mod public;
pub mod security; // Expose security module for CORS

// API documentation (always available)
pub mod docs;

use axum::{ routing::get, Router, http::Method };
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::infrastructure::container::DomainContainer;


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
      HeaderName::from_static("x-wallet-address"),   // Keep lowercase for CORS compatibility
      HeaderName::from_static("x-chain-id"),         // Keep lowercase for CORS compatibility
      HeaderName::from_static("x-web3-signature"),   // Standardized naming
      HeaderName::from_static("x-signed-message"),   // Standardized naming  
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

/// Create the main application router with unified architecture
/// Single source of truth - eliminates all route duplication and competing router systems
pub fn create_router(container: Arc<DomainContainer>) -> Router {
  // Use unified route builder - consolidates all 3 previous router systems
  routes::UnifiedRouteBuilder::new(container.clone())
    .build()
}

/// Create a demo router for Cloud Run demonstration without database dependencies
pub async fn create_demo_router() -> Router {
  use axum::response::Json;
  use serde_json::json;

  Router::new()
    // Health endpoint (demo mode - no external service checks)
    .route("/health", get(|| async {
      Json(
        json!({
          "status": "healthy",
          "service": "epsx-backend",
          "mode": "demo",
          "timestamp": chrono::Utc::now(),
          "services": {
            "postgres": {"status": "not_configured", "latency_ms": null},
            "redis": {"status": "not_configured", "latency_ms": null}
          }
        })
      )
    }))
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
    .layer(configure_cors_for_frontend())
}
