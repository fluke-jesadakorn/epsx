pub mod eps_handlers;

use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

use crate::web::AppState;
use crate::infra::services::tradingview::{TradingViewService, TradingViewApiService};
use crate::infra::InfraFactory;
use crate::config::Config;

pub use eps_handlers::*;

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub granularity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_connections: u32,
    pub requests_per_minute: u32,
    pub response_time_avg: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsData {
    pub user_growth: Vec<DataPoint>,
    pub trading_volume: Vec<DataPoint>,
    pub popular_symbols: Vec<SymbolData>,
    pub user_activity: Vec<ActivityData>,
}

#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub timestamp: String,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct SymbolData {
    pub symbol: String,
    pub volume: f64,
    pub count: u32,
}

#[derive(Debug, Serialize)]
pub struct ActivityData {
    pub hour: u8,
    pub active_users: u32,
    pub transactions: u32,
}

#[derive(Debug, Serialize)]
pub struct RealtimeMetrics {
    pub active_users: u32,
    pub concurrent_trades: u32,
    pub websocket_connections: u32,
    pub api_requests_per_second: f64,
    pub database_connections: u32,
    pub cache_hit_rate: f64,
    pub queue_size: u32,
}

#[derive(Debug, Serialize)]
pub struct RevenueAnalytics {
    pub total_revenue: f64,
    pub revenue_by_period: Vec<DataPoint>,
    pub revenue_by_product: Vec<ProductRevenue>,
    pub subscription_metrics: SubscriptionMetrics,
    pub payment_methods: Vec<PaymentMethodData>,
}

#[derive(Debug, Serialize)]
pub struct ProductRevenue {
    pub product_name: String,
    pub revenue: f64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionMetrics {
    pub active_subscriptions: u32,
    pub new_subscriptions: u32,
    pub churned_subscriptions: u32,
    pub mrr: f64,
    pub arr: f64,
}

#[derive(Debug, Serialize)]
pub struct PaymentMethodData {
    pub method: String,
    pub count: u32,
    pub total_amount: f64,
}

pub async fn get_system_metrics(
    Query(_params): Query<AnalyticsQuery>
) -> Result<Json<Value>, StatusCode> {
    let metrics = SystemMetrics {
        cpu_usage: 45.2,
        memory_usage: 62.8,
        active_connections: 156,
        requests_per_minute: 1247,
        response_time_avg: 89.3,
        error_rate: 0.12,
        uptime_seconds: 86400,
    };

    Ok(Json(json!({
        "status": "success",
        "data": metrics,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_analytics_data(
    Query(_params): Query<AnalyticsQuery>
) -> Result<Json<Value>, StatusCode> {
    let analytics = AnalyticsData {
        user_growth: vec![
            DataPoint { timestamp: "2024-01-01T00:00:00Z".to_string(), value: 1000.0 },
            DataPoint { timestamp: "2024-01-02T00:00:00Z".to_string(), value: 1150.0 },
            DataPoint { timestamp: "2024-01-03T00:00:00Z".to_string(), value: 1320.0 },
        ],
        trading_volume: vec![
            DataPoint { timestamp: "2024-01-01T00:00:00Z".to_string(), value: 50000.0 },
            DataPoint { timestamp: "2024-01-02T00:00:00Z".to_string(), value: 65000.0 },
            DataPoint { timestamp: "2024-01-03T00:00:00Z".to_string(), value: 58000.0 },
        ],
        popular_symbols: vec![
            SymbolData { symbol: "AAPL".to_string(), volume: 15000.0, count: 245 },
            SymbolData { symbol: "TSLA".to_string(), volume: 12000.0, count: 198 },
            SymbolData { symbol: "GOOGL".to_string(), volume: 9500.0, count: 156 },
        ],
        user_activity: vec![
            ActivityData { hour: 9, active_users: 450, transactions: 123 },
            ActivityData { hour: 10, active_users: 520, transactions: 156 },
            ActivityData { hour: 11, active_users: 680, transactions: 201 },
        ],
    };

    Ok(Json(json!({
        "status": "success",
        "data": analytics,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_realtime_metrics() -> Result<Json<Value>, StatusCode> {
    let metrics = RealtimeMetrics {
        active_users: 342,
        concurrent_trades: 28,
        websocket_connections: 156,
        api_requests_per_second: 45.7,
        database_connections: 12,
        cache_hit_rate: 0.94,
        queue_size: 3,
    };

    Ok(Json(json!({
        "status": "success",
        "data": metrics,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_revenue_analytics(
    Query(_params): Query<AnalyticsQuery>
) -> Result<Json<Value>, StatusCode> {
    let revenue = RevenueAnalytics {
        total_revenue: 125000.0,
        revenue_by_period: vec![
            DataPoint { timestamp: "2024-01-01T00:00:00Z".to_string(), value: 35000.0 },
            DataPoint { timestamp: "2024-01-02T00:00:00Z".to_string(), value: 42000.0 },
            DataPoint { timestamp: "2024-01-03T00:00:00Z".to_string(), value: 48000.0 },
        ],
        revenue_by_product: vec![
            ProductRevenue { product_name: "Premium Analytics".to_string(), revenue: 75000.0, percentage: 60.0 },
            ProductRevenue { product_name: "API Access".to_string(), revenue: 30000.0, percentage: 24.0 },
            ProductRevenue { product_name: "Custom Reports".to_string(), revenue: 20000.0, percentage: 16.0 },
        ],
        subscription_metrics: SubscriptionMetrics {
            active_subscriptions: 1250,
            new_subscriptions: 87,
            churned_subscriptions: 23,
            mrr: 45000.0,
            arr: 540000.0,
        },
        payment_methods: vec![
            PaymentMethodData { method: "Credit Card".to_string(), count: 450, total_amount: 85000.0 },
            PaymentMethodData { method: "PayPal".to_string(), count: 180, total_amount: 25000.0 },
            PaymentMethodData { method: "Crypto".to_string(), count: 95, total_amount: 15000.0 },
        ],
    };

    Ok(Json(json!({
        "status": "success",
        "data": revenue,
        "timestamp": chrono::Utc::now()
    })))
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
        .route("/analytics/system/metrics", get(get_system_metrics))
        .route("/analytics/data", get(get_analytics_data))
        .route("/analytics/realtime", get(get_realtime_metrics))
        .route("/analytics/revenue", get(get_revenue_analytics))
        // EPS Analytics endpoints - using proper handlers with service injection
        .route("/analytics/eps-rankings", get(eps_handlers::get_eps_rankings))
        .route("/analytics/eps-rankings/countries", get(eps_handlers::get_available_countries))
        .route("/analytics/eps-rankings/countries/all", get(eps_handlers::get_all_valid_countries))
        .route("/analytics/eps-rankings/sectors", get(eps_handlers::get_sectors_by_country))
        .route("/analytics/eps-rankings/health", get(eps_handlers::eps_health_check))
        .route("/analytics/eps-rankings/sync", post(eps_handlers::trigger_eps_sync))
        .route("/analytics/eps-rankings/websocket-debug", post(eps_handlers::debug_websocket_eps))
        // Live cache-based card dashboard endpoint (PRIMARY)
        .route("/analytics/rankings", get(eps_handlers::get_unified_analytics_rankings_cached))
        // Legacy database-based endpoint
        .route("/analytics/rankings/legacy", get(eps_handlers::get_unified_analytics_rankings))
        // Cache management endpoints
        .route("/analytics/cache/stats", get(eps_handlers::get_cache_stats))
        .route("/analytics/cache/refresh", post(eps_handlers::force_cache_refresh))
        .route("/analytics/cache/health", get(eps_handlers::cache_health_check))
        // Add services as extensions
        .layer(Extension(eps_ranking_service))
        .layer(Extension(eps_cache_service))
}

/// Real EPS rankings endpoint using TradingView data
pub async fn real_eps_rankings(
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Json<Value>, StatusCode> {
    info!("Fetching real EPS rankings from TradingView");
    
    let page = params.get("page").and_then(|p| p.parse::<i32>().ok());
    let limit = params.get("limit").and_then(|l| l.parse::<i32>().ok());
    let country = params.get("country").cloned();
    let use_websocket = params.get("websocket")
        .and_then(|ws| ws.parse::<bool>().ok())
        .unwrap_or(false);
    
    // Create TradingView service
    let config = Arc::new(Config::from_env());
    let tradingview_service = TradingViewApiService::new(config);
    
    // Use enhanced EPS rankings if WebSocket is requested
    let result = if use_websocket {
        tradingview_service.fetch_enhanced_eps_rankings(page, limit, country, true).await
    } else {
        tradingview_service.fetch_eps_rankings_for_frontend(page, limit, country).await
    };
    
    match result {
        Ok(response) => {
            info!("Successfully fetched {} EPS rankings (websocket: {})", response.data.len(), use_websocket);
            Ok(Json(serde_json::to_value(response).unwrap_or_else(|e| {
                error!("Failed to serialize response: {}", e);
                json!({"error": "Failed to serialize response"})
            })))
        }
        Err(e) => {
            error!("Failed to fetch EPS rankings: {}", e);
            
            // Return fallback data on error
            let page = page.unwrap_or(1);
            let limit = limit.unwrap_or(10);
            let fallback_response = json!({
                "data": [],
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": 0,
                    "totalPages": 0,
                    "hasNext": false,
                    "hasPrev": false
                },
                "error": format!("TradingView service error: {}", e)
            });
            
            Ok(Json(fallback_response))
        }
    }
}

/// Real countries endpoint with TradingView markets
pub async fn real_eps_countries() -> Result<Json<Value>, StatusCode> {
    let countries = vec![
        "america", "argentina", "australia", "austria", "bahrain", "bangladesh",
        "belgium", "brazil", "canada", "chile", "china", "colombia", "cyprus",
        "czech", "denmark", "egypt", "estonia", "finland", "france", "germany",
        "greece", "hongkong", "hungary", "iceland", "india", "indonesia",
        "ireland", "israel", "italy", "japan", "kenya", "kuwait", "latvia",
        "lithuania", "luxembourg", "malaysia", "mexico", "morocco", "netherlands",
        "newzealand", "nigeria", "norway", "pakistan", "peru", "philippines",
        "poland", "portugal", "qatar", "romania", "russia", "ksa", "serbia",
        "singapore", "slovakia", "rsa", "korea", "spain", "srilanka", "sweden",
        "switzerland", "taiwan", "thailand", "tunisia", "turkey", "uae", "uk",
        "venezuela", "vietnam"
    ];
    
    Ok(Json(json!({
        "countries": countries,
        "count": countries.len()
    })))
}

/// All countries endpoint (same as countries endpoint)  
pub async fn real_eps_all_countries() -> Result<Json<Value>, StatusCode> {
    // Same as regular countries endpoint - return all available markets
    real_eps_countries().await
}

/// Placeholder sectors endpoint
pub async fn placeholder_eps_sectors(
    Query(params): Query<std::collections::HashMap<String, String>>
) -> Result<Json<Value>, StatusCode> {
    let _country = params.get("country");
    
    Ok(Json(json!({
        "sectors": ["Technology", "Healthcare", "Financial Services", "Consumer Goods", "Energy", "Industrial"],
        "count": 6,
        "country": _country
    })))
}

/// Placeholder health check endpoint
pub async fn placeholder_eps_health() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "message": "EPS analytics service is operational (placeholder mode)",
        "available_countries": 13
    })))
}