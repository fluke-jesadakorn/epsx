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
///
/// `async` since wave 11 — the two new payments ports are
/// built from the payments pool here, which is an async
/// operation.
pub async fn create_router(
    container: Arc<DomainContainer>,
    notification_port: Option<Arc<dyn epsx_contracts::notification_port::NotificationPort>>,
) -> Router {
  // Use unified route builder - consolidates all 3 previous router systems
  let mut builder = routes::UnifiedRouteBuilder::new(container.clone())
    .with_notification_port(notification_port);

  // wave11(track-b): wire the two new payment-bounded-context
  // ports. The in-process adapters are built here from the
  // payments pool; the build is async because the
  // `get_payments_pool` call is. A missing pool means the
  // handlers fall back to 503 / empty (no panic, no silent
  // failure). The sync router path uses the
  // `with_*_port_opt` builder methods so a missing pool is
  // propagated as `None`.
  let payment_context_port: Option<Arc<dyn crate::domain::payment::repository_ports::PaymentContextRepositoryPort>> = {
      use crate::infrastructure::adapters::repositories::payment_context_repository_adapter::PaymentContextRepositoryAdapter;
      match crate::infrastructure::database::get_payments_pool().await {
          Ok(pool) => Some(Arc::new(PaymentContextRepositoryAdapter::new(pool)) as Arc<dyn crate::domain::payment::repository_ports::PaymentContextRepositoryPort>),
          Err(e) => {
              tracing::warn!(
                  "PaymentContextRepositoryPort NOT wired ({}); \
                   /api/public/payment-links/{{slug}} will return 503 \
                   and the admin CRUD endpoints will return 503.",
                  e
              );
              None
          }
      }
  };
  let subscription_port: Option<Arc<dyn crate::domain::payment::repository_ports::SubscriptionRepositoryPort>> = {
      use crate::infrastructure::adapters::repositories::payment::PaymentSubscriptionRepositoryAdapter;
      match crate::infrastructure::database::get_payments_pool().await {
          Ok(pool) => Some(Arc::new(PaymentSubscriptionRepositoryAdapter::new(pool)) as Arc<dyn crate::domain::payment::repository_ports::SubscriptionRepositoryPort>),
          Err(e) => {
              tracing::warn!(
                  "SubscriptionRepositoryPort NOT wired ({}); \
                   the market_analytics stock-ranking-assignments \
                   query will return an empty result.",
                  e
              );
              None
          }
      }
  };

  builder = builder
    .with_payment_context_repository_port(payment_context_port)
    .with_subscription_repository_port(subscription_port);

  builder.build()
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
