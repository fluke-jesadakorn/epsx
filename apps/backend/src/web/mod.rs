// Web layer implementation

pub mod auth;
pub mod oidc;
pub mod admin;
pub mod permission_profile;
pub mod user;
pub mod middleware;
pub mod modules;
pub mod validation;
pub mod health;
pub mod analytics;
pub mod settings;
pub mod templates;
pub mod admin_assignment;

use axum::{
    routing::{get, post},
    Router,
    response::Json,
    http::Method,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::infra::AppContainer;
use auth::AppState;

/// Health check handler
pub async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "service": "epsx-backend"
    }))
}

/// Cache handler (placeholder)
pub async fn cache_handler() -> Json<Value> {
    Json(json!({
        "status": "cache_cleared",
        "timestamp": chrono::Utc::now()
    }))
}

/// Premium rankings handler (placeholder)
pub async fn premium_rankings_handler() -> Json<Value> {
    Json(json!({
        "rankings": [],
        "last_updated": chrono::Utc::now()
    }))
}




/// Placeholder handlers for payment endpoints
#[allow(dead_code)]
async fn placeholder_crypto_deposit() -> Json<Value> {
    Json(json!({
        "message": "Crypto deposit address endpoint - implementation pending",
        "address": null
    }))
}

#[allow(dead_code)]
async fn placeholder_musepay_create() -> Json<Value> {
    Json(json!({
        "message": "MusePay create payment endpoint - implementation pending",
        "payment_id": null
    }))
}

#[allow(dead_code)]
async fn placeholder_musepay_webhook() -> Json<Value> {
    Json(json!({
        "message": "MusePay webhook processed",
        "status": "received"
    }))
}

/// Placeholder handler for notification endpoints
#[allow(dead_code)]
async fn placeholder_notification_handler() -> Json<Value> {
    Json(json!({
        "message": "Notification endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for monitoring endpoints
#[allow(dead_code)]
async fn placeholder_monitoring_handler() -> Json<Value> {
    Json(json!({
        "message": "Monitoring endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Placeholder handler for stream endpoints
#[allow(dead_code)]
async fn placeholder_stream_handler() -> Json<Value> {
    Json(json!({
        "message": "Stream endpoint not yet implemented",
        "status": "placeholder",
        "timestamp": chrono::Utc::now()
    }))
}

/// Configure CORS for frontend applications
fn configure_cors_for_frontend() -> CorsLayer {
    use crate::config::get_env_var;
    use tower_http::cors::AllowOrigin;
    use http::{HeaderName, HeaderValue};
    
    if get_env_var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "development" {
        // Development - allow common development origins
        let dev_origins = vec![
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),  // Frontend dev server
            "http://localhost:3001".parse::<HeaderValue>().unwrap(),  // Admin frontend dev server
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),  // Alternative localhost
            "http://127.0.0.1:3001".parse::<HeaderValue>().unwrap(),  // Alternative localhost
        ];
        
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(dev_origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
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
            "https://api.epsx.io",
            // Google Cloud Run domains
            "https://epsx-frontend-epsx-service-run.app",
            "https://epsx-admin-epsx-service-run.app", 
            "https://epsx-backend-epsx-service-run.app",
            // Generic Cloud Run patterns for us-central1
            "https://epsx-frontend-1234567890-uc.a.run.app",
            "https://epsx-admin-1234567890-uc.a.run.app",
            "https://epsx-backend-1234567890-uc.a.run.app",
            // Vercel deployment domains (legacy)
            "https://epsx-frontend.vercel.app",
            "https://epsx-admin.vercel.app",
            "https://epsx-backend.vercel.app"
        ];
        
        for domain in production_domains {
            if let Ok(origin) = domain.parse() {
                allowed_origins.push(origin);
            }
        }
        
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed_origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
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
async fn create_standalone_analytics_routes(infra_factory: &crate::infra::InfraFactory) -> Router {
    use axum::Extension;
    
    // Create services for analytics
    let eps_ranking_service = infra_factory.create_eps_ranking_service();
    
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
                },
                rate_limiting: crate::config::RateLimitingConfig {
                    default_per_minute: 60,
                    endpoint_specific: std::collections::HashMap::new(),
                },
            })
        }
    };
    let tradingview_service = std::sync::Arc::new(crate::infra::services::tradingview::TradingViewApiService::new(config));
    let eps_repository = infra_factory.create_eps_repo();
    let eps_cache_service = std::sync::Arc::new(
        crate::dom::services::eps_cache_service::EPSCacheService::new(
            tradingview_service,
            eps_repository,
            None // Use default cache config
        )
    );

    // Create unified cache service (automatically selects InMemory or Redis)
    let unified_cache_service = match crate::infra::cache::CacheFactory::from_env().await {
        Ok(cache) => cache,
        Err(e) => {
            tracing::warn!("Failed to create cache service: {}, falling back to in-memory cache", e);
            // Fallback to in-memory cache with default config
            std::sync::Arc::new(crate::infra::cache::InMemoryCache::new(
                crate::infra::cache::CacheConfig::default()
            )) as std::sync::Arc<dyn crate::infra::cache::Cache>
        }
    };

    // Background cache refresh removed - using on-demand loading instead
    
    Router::new()
        // Primary analytics ranking endpoints (using corrected FQ field logic)
        .route("/api/v1/analytics/rankings", get(analytics::eps_handlers::get_unified_analytics_rankings_cached))
        .route("/api/v1/analytics/eps-rankings", get(analytics::eps_handlers::get_unified_analytics_rankings_cached))
        .route("/api/v1/analytics/eps-rankings/countries", get(analytics::eps_handlers::get_available_countries))
        .route("/api/v1/analytics/eps-rankings/countries/all", get(analytics::eps_handlers::get_all_valid_countries))
        .route("/api/v1/analytics/eps-rankings/sectors", get(analytics::eps_handlers::get_sectors_by_country))
        .route("/api/v1/analytics/eps-rankings/health", get(analytics::eps_handlers::eps_health_check))
        .route("/api/v1/analytics/eps-rankings/sync", post(analytics::eps_handlers::trigger_eps_sync))
        .route("/api/v1/analytics/eps-rankings/websocket-debug", post(analytics::eps_handlers::debug_websocket_eps))
        .route("/api/v1/analytics/eps-rankings/debug-eps-correction", post(analytics::eps_handlers::debug_eps_correction))
        .route("/api/v1/analytics/eps-rankings/debug-ranking-data", post(analytics::eps_handlers::debug_ranking_data))
        // Add cache endpoints
        .route("/api/v1/analytics/cache/stats", get(analytics::eps_handlers::get_cache_stats))
        .route("/api/v1/analytics/cache/refresh", post(analytics::eps_handlers::force_cache_refresh))
        .route("/api/v1/analytics/cache/health", get(analytics::eps_handlers::cache_health_check))
        // Add services as extensions
        .layer(Extension(eps_ranking_service))
        .layer(Extension(eps_cache_service))
        .layer(Extension(unified_cache_service))
}

/// Create the main application router with analytics support
pub async fn create_router(container: Arc<AppContainer>) -> Router {
    // Create OIDC routes with full functionality including POST handlers
    let app_state = container.create_app_state();
    let oidc_routes = oidc::routes::oidc_routes().with_state(app_state);
    
    // Create analytics routes that use the container's InfraFactory
    let analytics_routes = create_standalone_analytics_routes(&container.infra).await;
    
    // Create core routes
    let core_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/cache", get(cache_handler));
    
    // Configure CORS for all routes
    let cors = configure_cors_for_frontend();
    
    // Merge routes with analytics support
    core_routes
        .merge(oidc_routes)
        .merge(analytics_routes)
        .layer(cors)
}

/// Create a demo router for Cloud Run demonstration without database dependencies
pub async fn create_demo_router() -> Router {
    use axum::response::Json;
    use serde_json::json;
    
    Router::new()
        .route("/health", get(health_handler))
        .route("/", get(|| async { Json(json!({
            "status": "running",
            "service": "epsx-backend",
            "mode": "demo",
            "message": "Backend is running in demo mode. Database connection required for full functionality.",
            "timestamp": chrono::Utc::now()
        })) }))
}

/// Create test application for integration tests
#[cfg(test)]
pub async fn create_test_app() -> Router {
    // Test router with v1 API structure only
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/auth/login", post(health_handler)) // Mock v1 endpoint
        .route("/api/v1/permission-profiles", get(health_handler)) // Mock v1 endpoint
        .route("/api/v1/auth/me", get(health_handler)) // Mock v1 endpoint
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn should_create_health_response() {
        let response = health_handler().await;
        let json_value = response.0;
        
        assert!(json_value.get("status").is_some());
        assert_eq!(json_value["status"], "healthy");
        assert!(json_value.get("timestamp").is_some());
        assert_eq!(json_value["service"], "epsx-backend");
    }
}