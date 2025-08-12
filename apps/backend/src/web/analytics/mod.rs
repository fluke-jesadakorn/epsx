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
    let config = std::sync::Arc::new(Config::from_env());
    let tradingview_service = std::sync::Arc::new(TradingViewApiService::new(config));
    let eps_repository = infra_factory.create_eps_repo();
    let eps_cache_service = std::sync::Arc::new(
        crate::dom::services::eps_cache_service::EPSCacheService::new(
            tradingview_service,
            eps_repository,
            None // Use default cache config
        )
    );

    // Start background cache refresh (spawn async task)
    let cache_service_clone = eps_cache_service.clone();
    tokio::spawn(async move {
        cache_service_clone.start_background_refresh().await;
    });
    
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
        // System metrics endpoint for admin dashboard
        .route("/analytics/system/metrics", get(system_metrics_handler))
        // Add services as extensions
        .layer(Extension(eps_ranking_service))
        .layer(Extension(eps_cache_service))
}

/// System metrics handler for admin dashboard
/// GET /api/v1/analytics/system/metrics
async fn system_metrics_handler(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Reuse the existing system metrics collection from health module
    let metrics = crate::web::health::casbin_health_check::collect_system_metrics(&app_state).await;
    
    // Format response to match frontend expectations
    let response = serde_json::json!({
        "status": "success",
        "data": metrics,
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(response))
}
