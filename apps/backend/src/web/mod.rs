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
// CRITICAL: Comprehensive Error System (Phase 1.3)
pub mod errors;
pub mod responses; // Unified API response format
pub mod api_response; // New Standard Response Format
pub mod public;
pub mod security; // Expose security module for CORS

// API documentation (always available)
pub mod docs;
pub mod pagination;

use axum::{ routing::get, Router };
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::infrastructure::container::DomainContainer;


/// Configure CORS for frontend applications - uses environment-aware security module
fn configure_cors_for_frontend() -> CorsLayer {
  security::cors::get_cors_layer()
}

// create_standalone_analytics_routes function removed
// Analytics routes are now handled by UnifiedRouteBuilder

/// Create the main application router with unified architecture
/// Single source of truth - eliminates all route duplication and competing router systems
pub fn create_router(
    container: Arc<DomainContainer>,
    notification_port: Option<Arc<dyn epsx_contracts::notification_port::NotificationPort>>,
) -> Router {
  // Use unified route builder - consolidates all 3 previous router systems
  routes::UnifiedRouteBuilder::new(container.clone())
    .with_notification_port(notification_port)
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
