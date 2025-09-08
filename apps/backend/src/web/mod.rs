// Web layer implementation

pub mod auth;
pub mod oidc;
pub mod admin;
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
pub mod realtime;
pub mod session_management_handlers;
pub mod session_management_routes;
pub mod notifications;

use axum::{ routing::{ get, post }, Router, response::Json, http::Method };
use serde_json::{ json, Value };
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use axum::middleware as axum_middleware;

use crate::infrastructure::AppContainer;

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


/// Configure CORS for frontend applications
fn configure_cors_for_frontend() -> CorsLayer {
  use crate::config::get_env_var;
  use tower_http::cors::AllowOrigin;
  use http::{ HeaderName, HeaderValue };

  if
    get_env_var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) ==
    "development"
  {
    // Development - allow common development origins
    let dev_origins = vec![
      "http://localhost:3000".parse::<HeaderValue>().unwrap(), // Frontend dev server
      "http://localhost:3001".parse::<HeaderValue>().unwrap(), // Admin frontend dev server
      "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(), // Alternative localhost
      "http://127.0.0.1:3001".parse::<HeaderValue>().unwrap() // Alternative localhost
    ];

    CorsLayer::new()
      .allow_origin(AllowOrigin::list(dev_origins))
      .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
      ])
      .allow_headers([
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-requested-with"),
        HeaderName::from_static("accept"),
        HeaderName::from_static("origin"),
        HeaderName::from_static("x-client-id"),
        HeaderName::from_static("x-api-key"),
      ])
      .allow_credentials(true)
  } else {
    // Production - only allow configured frontend URLs
    let mut allowed_origins = vec![];

    if let Ok(frontend_url) = get_env_var("FRONTEND_URL") {
      if let Ok(origin) = frontend_url.parse() {
        allowed_origins.push(origin);
      }
    }

    if let Ok(admin_url) = get_env_var("ADMIN_FRONTEND_URL") {
      if let Ok(origin) = admin_url.parse() {
        allowed_origins.push(origin);
      }
    }

    if let Ok(prod_frontend) = get_env_var("PRODUCTION_FRONTEND_URL") {
      if let Ok(origin) = prod_frontend.parse() {
        allowed_origins.push(origin);
      }
    }

    if let Ok(prod_admin) = get_env_var("PRODUCTION_ADMIN_URL") {
      if let Ok(origin) = prod_admin.parse() {
        allowed_origins.push(origin);
      }
    }

    // Allow any *.run.app domain for Cloud Run deployments
    if get_env_var("RUST_ENV").unwrap_or_default() == "production" {
      // Note: This is handled by the dynamic domains list below
      // Could add more dynamic handling here if needed
    }

    // Add production deployment domains
    let production_domains = vec![
      "https://epsx.io",
      "https://www.epsx.io",
      "https://admin.epsx.io",
      "https://api.epsx.io"
    ];

    for domain in production_domains {
      if let Ok(origin) = domain.parse() {
        allowed_origins.push(origin);
      }
    }

    CorsLayer::new()
      .allow_origin(AllowOrigin::list(allowed_origins))
      .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
      ])
      .allow_headers([
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-requested-with"),
        HeaderName::from_static("accept"),
        HeaderName::from_static("origin"),
        HeaderName::from_static("x-client-id"),
        HeaderName::from_static("x-api-key"),
      ])
      .allow_credentials(true)
  }
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
      // Use a minimal config that should work for basic operation
      std::sync::Arc::new(crate::config::Config {
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
  let config = match crate::config::Config::from_env() {
    Ok(config) => std::sync::Arc::new(config),
    Err(_) => {
      // Use minimal config for DDD migration
      std::sync::Arc::new(crate::config::Config {
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
          supported_currencies: vec!["USD".to_string()],
          default_currency: "USD".to_string(),
          default_checkout_url_template: "https://localhost:3000/checkout/{}".to_string(),
        },
        email: crate::config::EmailConfig {
          from_email: "noreply@localhost".to_string(),
          from_name: "EPSX".to_string(),
          sendgrid_api_key: "mock-key".to_string(),
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
      })
    }
  };

  // Create OIDC routes with full functionality including POST handlers  
  let app_state = container.create_app_state().await
    .map_err(|e| {
      tracing::error!("Failed to create app state: {}", e);
      e
    })?;
  let oidc_routes = oidc::routes::oidc_routes().with_state(app_state.clone());
  

  // Performance routes removed - stub implementations cleaned up

  // Permission routes removed - replaced by simple roles middleware

  // Notification routes removed - will be re-implemented

  // Create admin routes (core admin functionality)
  let admin_routes = admin::routes::create_admin_routes().with_state(app_state.clone());
  let admin_public_routes = admin::routes::create_admin_public_routes().with_state(app_state.clone());

  // Create realtime routes for SSE
  let realtime_routes = realtime::routes::create_realtime_routes().with_state(app_state.clone());


  // Create analytics routes with permission middleware
  let analytics_routes = analytics::create_analytics_router(&container.infra).await;
  
  // FCM routes removed - will be re-implemented


  // Create core routes
  let core_routes = Router::new()
    .route("/health", get(health_handler))
    .route("/cache", get(cache_handler))
    .with_state(container.clone());

  // Configure CORS for all routes
  let cors = configure_cors_for_frontend();

  // Merge routes with analytics, admin, and permissions support
  Ok(core_routes
    .merge(oidc_routes)
    .merge(realtime_routes)
    .merge(analytics_routes)
    .nest("/api/v1/notifications", notifications::notification_routes()
      // DDD Notification infrastructure adapter - bridges legacy services with DDD bounded context
      .layer(axum::Extension(Arc::new(crate::infrastructure::adapters::repositories::NotificationRepositoryAdapter::new(
        Arc::new(crate::infrastructure::adapters::services::fcm_service::FcmService::new(container.infra.firebase_admin.clone())),
        // Create stub email service for DDD migration - will be replaced with proper service
        Arc::new(crate::infrastructure::adapters::services::email_service::SendGridEmailService::new(
          config.email.sendgrid_api_key.clone(),
        )),
      ))))
      // Keep legacy services for endpoints that haven't been migrated yet
      .layer(axum::Extension(container.fcm_topic_service.clone()))
      .layer(axum::Extension(container.user_notification_repo.clone()))
      .layer(axum_middleware::from_fn_with_state(
        app_state.clone(),
        crate::web::middleware::clean_auth_middleware
      ))
    )
    .nest("/api/v1/admin", admin_routes
      .layer(axum::Extension(container.fcm_service.clone()))
      .layer(axum::Extension(container.fcm_topic_service.clone()))
      .layer(axum::Extension(container.user_notification_repo.clone()))
    )
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
    .layer(axum_middleware::from_fn(
      crate::web::middleware::enhanced_cors_middleware
    ))
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
