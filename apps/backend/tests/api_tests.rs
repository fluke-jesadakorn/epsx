// API integration tests for EPSX backend
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use epsx::web::create_test_app;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_endpoints() {
    let app = create_test_app().await;
    
    // Test login endpoint exists
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({
                    "email": "test@example.com",
                    "password": "password123"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should return either success or validation error, not 404
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_template_endpoints() {
    let app = create_test_app().await;
    
    // Test templates list endpoint (should require auth)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/templates")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should return either success, unauthorized, or not found
    // But not an internal server error
    assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_user_endpoints() {
    let app = create_test_app().await;
    
    // Test user profile endpoint (should require auth)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/user/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should return unauthorized for unauthenticated request
    assert!(matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::NOT_FOUND
    ));
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/auth/login")
                .header("Origin", "http://localhost:3000")
                .header("Access-Control-Request-Method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // CORS preflight should return OK
    assert_eq!(response.status(), StatusCode::OK);
}

// Mock data helpers for testing
pub fn create_test_user_payload() -> serde_json::Value {
    json!({
        "email": "test@example.com",
        "password": "password123",
        "display_name": "Test User"
    })
}

pub fn create_test_template_payload() -> serde_json::Value {
    json!({
        "name": "Test Template",
        "description": "A test role template",
        "target_tier": "bronze",
        "category": "user",
        "permissions": [
            {
                "action": "read",
                "resource": "posts",
                "conditions": null
            }
        ],
        "tags": ["test", "basic"],
        "metadata": {
            "prerequisites": [],
            "warnings": [],
            "use_cases": ["testing"],
            "max_assignments": 100,
            "requires_approval": false,
            "auto_expire_days": null,
            "custom_fields": {}
        }
    })
}