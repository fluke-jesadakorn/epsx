pub mod eps_handlers;

use axum::{
    routing::{get, post},
    Router,
    Extension,
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::web::AppState;
use crate::infra::services::tradingview::TradingViewApiService;
use crate::infra::InfraFactory;
use crate::config::Config;

pub use eps_handlers::*;

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub granularity: Option<String>,
}


pub fn create_analytics_router(infra_factory: &InfraFactory) -> Router<AppState> {
    // Create services for both database and cache approaches
    let eps_ranking_service = infra_factory.create_eps_ranking_service();
    
    // Create cache-based EPS service with TradingView integration
    let config = match Config::from_env() {
        Ok(config) => std::sync::Arc::new(config),
        Err(e) => {
            tracing::warn!("Failed to load config, using fallback: {:?}", e);
            // Use a minimal config that should work for basic operation
            std::sync::Arc::new(Config {
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
                    nextauth_secret: "default-nextauth-secret".to_string(),
                    jwt_secret: "default-jwt-secret".to_string(),
                    cookie_signing_key: None,
                    cookie_encryption_key: None,
                    firebase_project_id: None,
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
                },
                rate_limiting: crate::config::RateLimitingConfig {
                    default_per_minute: 60,
                    endpoint_specific: std::collections::HashMap::new(),
                },
            })
        }
    };
    let tradingview_service = std::sync::Arc::new(TradingViewApiService::new(config));
    let eps_repository = infra_factory.create_eps_repo();
    let eps_cache_service = std::sync::Arc::new(
        crate::dom::services::eps_cache_service::EPSCacheService::new(
            tradingview_service,
            eps_repository,
            None // Use default cache config
        )
    );

    // Background cache refresh removed - using on-demand loading instead
    
    Router::new()
        // Primary analytics ranking endpoint (preserved)
        .route("/analytics/rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        // EPS Analytics endpoints - using proper handlers with service injection
        .route("/analytics/eps-rankings", get(eps_handlers::get_eps_rankings))
        .route("/analytics/eps-rankings/countries", get(eps_handlers::get_available_countries))
        .route("/analytics/eps-rankings/countries/all", get(eps_handlers::get_all_valid_countries))
        .route("/analytics/eps-rankings/sectors", get(eps_handlers::get_sectors_by_country))
        .route("/analytics/eps-rankings/health", get(eps_handlers::eps_health_check))
        .route("/analytics/eps-rankings/sync", post(eps_handlers::trigger_eps_sync))
        .route("/analytics/eps-rankings/websocket-debug", post(eps_handlers::debug_websocket_eps))
        .route("/analytics/eps-rankings/debug-eps-correction", post(eps_handlers::debug_eps_correction))
        .route("/analytics/eps-rankings/debug-ranking-data", post(eps_handlers::debug_ranking_data))
        // System metrics endpoint for admin dashboard
        .route("/analytics/system/metrics", get(system_metrics_handler))
        // Add services as extensions
        .layer(Extension(eps_ranking_service))
        .layer(Extension(eps_cache_service))
}

/// System metrics handler for admin dashboard
/// GET /api/v1/analytics/system/metrics
async fn system_metrics_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Reuse the existing system metrics collection from health module
    // let metrics = crate::web::health::casbin_health_check::collect_system_metrics(&app_state).await; // Removed Casbin
    let metrics: std::collections::HashMap<String, String> = std::collections::HashMap::new(); // Placeholder for metrics
    
    // Format response to match frontend expectations
    let response = serde_json::json!({
        "status": "success",
        "data": metrics,
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(response))
}
