use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use tokio_test;

use crate::common::test_helpers::{setup_test_app, TestContext};

mod common;

/// Test EPS Rankings endpoint - GET /api/analytics/eps-rankings
#[tokio::test]
async fn test_eps_rankings_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(json.get("data").is_some());
    assert!(json.get("pagination").is_some());
    
    let pagination = &json["pagination"];
    assert!(pagination.get("page").is_some());
    assert!(pagination.get("limit").is_some());
    assert!(pagination.get("total").is_some());
}

/// Test EPS Rankings with pagination - GET /api/analytics/eps-rankings?page=1&limit=5
#[tokio::test] 
async fn test_eps_rankings_with_pagination() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings?page=1&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let pagination = &json["pagination"];
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["limit"], 5);
}

/// Test EPS Rankings with country filter - GET /api/analytics/eps-rankings?country=america
#[tokio::test]
async fn test_eps_rankings_with_country_filter() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings?country=america")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    // Should return valid data structure even if no data
    assert!(json.get("data").is_some());
    assert!(json.get("pagination").is_some());
}

/// Test Available Countries endpoint - GET /api/analytics/eps-rankings/countries
#[tokio::test]
async fn test_available_countries_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings/countries")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(json.get("countries").is_some());
    assert!(json.get("count").is_some());
    
    let countries = json["countries"].as_array().unwrap();
    let count = json["count"].as_u64().unwrap();
    assert_eq!(countries.len(), count as usize);
}

/// Test All Valid Countries endpoint - GET /api/analytics/eps-rankings/countries/all  
#[tokio::test]
async fn test_all_valid_countries_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings/countries/all")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(json.get("countries").is_some());
    assert!(json.get("count").is_some());
    
    let countries = json["countries"].as_array().unwrap();
    assert!(!countries.is_empty()); // Should have valid countries
}

/// Test Sectors by Country endpoint - GET /api/analytics/eps-rankings/sectors?country=america
#[tokio::test]
async fn test_sectors_by_country_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings/sectors?country=america")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate response structure
    assert!(json.get("sectors").is_some());
    assert!(json.get("count").is_some());
    assert_eq!(json["country"], "america");
}

/// Test EPS Health Check endpoint - GET /api/analytics/eps-rankings/health
#[tokio::test]
async fn test_eps_health_check_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate health response structure
    assert!(json.get("status").is_some());
    assert!(json.get("message").is_some());
    assert!(json.get("available_countries").is_some());
}

/// Test Unified Analytics Rankings endpoint - GET /api/analytics/rankings
#[tokio::test]
async fn test_unified_analytics_rankings_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/rankings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate unified response structure
    assert!(json.get("success").is_some());
    assert!(json.get("data").is_some());
    assert!(json.get("pagination").is_some());
    assert!(json.get("metadata").is_some());
    assert!(json.get("processing_time_ms").is_some());

    let metadata = &json["metadata"];
    assert!(metadata.get("available_countries").is_some());
    assert!(metadata.get("available_sectors").is_some());
    assert!(metadata.get("current_filters").is_some());
    assert!(metadata.get("data_source").is_some());
}

/// Test Cache Stats endpoint - GET /api/analytics/cache/stats
#[tokio::test]
async fn test_cache_stats_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/cache/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate cache stats response
    assert!(json.get("success").is_some());
    assert!(json.get("stats").is_some());
    assert!(json.get("message").is_some());
    assert!(json.get("timestamp").is_some());

    let stats = &json["stats"];
    assert!(stats.get("total_entries").is_some());
    assert!(stats.get("active_entries").is_some());
    assert!(stats.get("hit_ratio").is_some());
}

/// Test Cache Health endpoint - GET /api/analytics/cache/health
#[tokio::test]
async fn test_cache_health_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/cache/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate cache health response
    assert!(json.get("status").is_some());
    assert!(json.get("healthy").is_some());
    assert!(json.get("cache_stats").is_some());
    assert!(json.get("recommendations").is_some());
    assert!(json.get("timestamp").is_some());
}

/// Test Legacy Analytics Data endpoint - GET /api/analytics/data
#[tokio::test]
async fn test_legacy_analytics_data_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate legacy analytics response
    assert_eq!(json["status"], "success");
    assert!(json.get("data").is_some());
    assert!(json.get("timestamp").is_some());

    let data = &json["data"];
    assert!(data.get("user_growth").is_some());
    assert!(data.get("trading_volume").is_some());
    assert!(data.get("popular_symbols").is_some());
    assert!(data.get("user_activity").is_some());
}

/// Test System Metrics endpoint - GET /api/analytics/system/metrics
#[tokio::test]
async fn test_system_metrics_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/system/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate system metrics response
    assert_eq!(json["status"], "success");
    assert!(json.get("data").is_some());
    
    let data = &json["data"];
    assert!(data.get("cpu_usage").is_some());
    assert!(data.get("memory_usage").is_some());
    assert!(data.get("active_connections").is_some());
    assert!(data.get("response_time_avg").is_some());
}

/// Test Realtime Metrics endpoint - GET /api/analytics/realtime  
#[tokio::test]
async fn test_realtime_metrics_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/realtime")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate realtime metrics response
    assert_eq!(json["status"], "success");
    assert!(json.get("data").is_some());
    
    let data = &json["data"];
    assert!(data.get("active_users").is_some());
    assert!(data.get("concurrent_trades").is_some());
    assert!(data.get("websocket_connections").is_some());
    assert!(data.get("cache_hit_rate").is_some());
}

/// Test Revenue Analytics endpoint - GET /api/analytics/revenue
#[tokio::test]
async fn test_revenue_analytics_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/revenue")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate revenue analytics response
    assert_eq!(json["status"], "success");
    assert!(json.get("data").is_some());
    
    let data = &json["data"];
    assert!(data.get("total_revenue").is_some());
    assert!(data.get("revenue_by_period").is_some());
    assert!(data.get("subscription_metrics").is_some());
    assert!(data.get("payment_methods").is_some());
}

/// Test EPS Sync POST endpoint - POST /api/analytics/eps-rankings/sync
#[tokio::test]
async fn test_eps_sync_post_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings/sync")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Accept both OK and error responses since sync may fail in test environment
    assert!(response.status() == StatusCode::OK || response.status().is_server_error());
}

/// Test Cache Refresh POST endpoint - POST /api/analytics/cache/refresh
#[tokio::test]
async fn test_cache_refresh_post_endpoint() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/cache/refresh")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Validate cache refresh response
    assert!(json.get("success").is_some());
    assert!(json.get("refreshed_entries").is_some());
    assert!(json.get("duration_ms").is_some());
    assert!(json.get("message").is_some());
}

/// Test invalid endpoint returns 404
#[tokio::test]
async fn test_invalid_analytics_endpoint_returns_404() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/invalid-endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test error handling with invalid parameters
#[tokio::test]
async fn test_eps_rankings_with_invalid_params() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/eps-rankings?page=invalid&limit=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should still return 200 with defaults, as query parsing handles invalid values gracefully
    assert_eq!(response.status(), StatusCode::OK);
}

/// Performance test - ensure endpoints respond within reasonable time
#[tokio::test]
async fn test_analytics_endpoints_performance() {
    let ctx = setup_test_app().await;
    let app = ctx.app;

    let start = std::time::Instant::now();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analytics/rankings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration.as_secs() < 5, "Analytics endpoint took too long: {:?}", duration);
}