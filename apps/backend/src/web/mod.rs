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
  // Use unified route builder - consolidates all 3 previous router systems.
  // Wave 11 / Track A: pull the PaymentRepositoryPort and
  // CreditRepositoryPort accessors from the container so the
  // 8 cross-pool handler collapses in `web/payments/*` have
  // a port to call. If the container wasn't initialized with
  // these (e.g. test harness), the AppState ends up with
  // `payment_repo = None` and the handlers panic-fast at
  // startup with a clear "port not wired" message rather
  // than silently falling back to the cross-pool path.
  // Wave 11 / Track B: also wire the two new
  // payment-bounded-context ports (PaymentContext + Subscription).
  let payment_repo = container.get_payment_repository_port();
  let credit_repo = container.get_credit_repository_port();
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
  routes::UnifiedRouteBuilder::new(container.clone())
    .with_notification_port(notification_port)
    .with_payment_repository_port(payment_repo)
    .with_credit_repository_port(credit_repo)
    .with_payment_context_repository_port(payment_context_port)
    .with_subscription_repository_port(subscription_port)
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
