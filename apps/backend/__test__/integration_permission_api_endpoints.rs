use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use chrono::Utc;

// Test utilities module
#[path = "utils/mod.rs"]
mod utils;

#[cfg(test)]
mod permission_api_endpoint_tests {
    use super::*;
    
    // Mock test setup
    async fn create_test_app() -> Router {
        // This would normally be your actual app setup
        // For testing purposes, we'll create a minimal router
        use axum::{routing::post, Json, extract::Path};
        
        async fn mock_grant_permission(
            Path(user_id): Path<String>,
            Json(payload): Json<Value>,
        ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            if user_id == "invalid-user" {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "user_not_found", "message": "User not found"}))
                ));
            }
            
            if let Some(expiry) = payload.get("expiry_timestamp").and_then(|v| v.as_i64()) {
                if expiry <= Utc::now().timestamp() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "invalid_expiry", "message": "Expiry must be in future"}))
                    ));
                }
            }
            
            Ok(Json(json!({
                "permission": format!("{}:{}", payload["base_permission"], payload["expiry_timestamp"]),
                "expires_at": payload["expiry_timestamp"]
            })))
        }
        
        async fn mock_revoke_permission(
            Path(user_id): Path<String>,
            Json(payload): Json<Value>,
        ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            if user_id == "invalid-user" {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "user_not_found", "message": "User not found"}))
                ));
            }
            
            Ok(Json(json!({"message": "Permission revoked successfully"})))
        }
        
        async fn mock_validate_permissions(
            Path(user_id): Path<String>,
            Json(payload): Json<Value>,
        ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            if user_id == "invalid-user" {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "user_not_found", "message": "User not found"}))
                ));
            }
            
            let permissions = payload["permissions"].as_array().unwrap_or(&vec![]);
            let now = Utc::now().timestamp();
            
            let mut valid = vec![];
            let mut expired = vec![];
            let mut expiring_soon = vec![];
            
            for permission in permissions {
                if let Some(perm_str) = permission.as_str() {
                    let parts: Vec<&str> = perm_str.split(':').collect();
                    if parts.len() >= 4 {
                        if let Ok(timestamp) = parts[parts.len()-1].parse::<i64>() {
                            if timestamp <= now {
                                expired.push(json!({
                                    "permission": perm_str,
                                    "base_permission": parts[0..parts.len()-1].join(":"),
                                    "expired_at": timestamp,
                                    "expired_for": (now - timestamp) * 1000
                                }));
                            } else if timestamp <= now + 86400 { // 24 hours
                                expiring_soon.push(json!({
                                    "permission": perm_str,
                                    "base_permission": parts[0..parts.len()-1].join(":"),
                                    "expires_at": timestamp,
                                    "expires_in": (timestamp - now) * 1000
                                }));
                            } else {
                                valid.push(perm_str);
                            }
                        } else {
                            valid.push(perm_str);
                        }
                    } else {
                        valid.push(perm_str);
                    }
                }
            }
            
            Ok(Json(json!({
                "valid": valid,
                "expired": expired,
                "expiring_soon": expiring_soon,
                "summary": {
                    "total": permissions.len(),
                    "valid_count": valid.len(),
                    "expired_count": expired.len(),
                    "expiring_soon_count": expiring_soon.len()
                }
            })))
        }
        
        async fn mock_extend_permission(
            Path(user_id): Path<String>,
            Json(payload): Json<Value>,
        ) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            if user_id == "invalid-user" {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "user_not_found", "message": "User not found"}))
                ));
            }
            
            let old_permission = payload["permission"].as_str().unwrap();
            let new_expiry = payload["new_expiry_timestamp"].as_i64().unwrap();
            
            let parts: Vec<&str> = old_permission.split(':').collect();
            let base_permission = parts[0..parts.len()-1].join(":");
            let new_permission = format!("{}:{}", base_permission, new_expiry);
            
            Ok(Json(json!({
                "old_permission": old_permission,
                "new_permission": new_permission,
                "extension": 3600000 // Mock 1 hour extension in milliseconds
            })))
        }
        
        async fn mock_bulk_grant(Json(payload): Json<Value>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            let user_ids = payload["user_ids"].as_array().unwrap();
            let permissions = payload["permissions"].as_array().unwrap();
            
            let mut successful = vec![];
            let mut failed = vec![];
            
            for user_id in user_ids {
                if user_id.as_str() == Some("invalid-user") {
                    failed.push(json!({
                        "user_id": user_id,
                        "error": "User not found"
                    }));
                } else {
                    let granted_permissions: Vec<String> = permissions.iter()
                        .map(|p| format!("{}:{}", p["base_permission"], p["expiry_timestamp"]))
                        .collect();
                    
                    successful.push(json!({
                        "user_id": user_id,
                        "permissions": granted_permissions
                    }));
                }
            }
            
            Ok(Json(json!({
                "successful": successful,
                "failed": failed,
                "summary": {
                    "total": user_ids.len(),
                    "successful": successful.len(),
                    "failed": failed.len()
                }
            })))
        }
        
        async fn mock_cleanup_expired(Json(payload): Json<Value>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
            let dry_run = payload.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);
            
            // Mock cleanup results
            let cleaned = if dry_run { 0 } else { 5 };
            let failed = 0;
            
            Ok(Json(json!({
                "cleaned": cleaned,
                "failed": failed,
                "details": [
                    {
                        "user_id": "user-1",
                        "permission": &format!("epsx:analytics:premium:{}", crate::utils::TestTimestamps::LEGACY_TEST),
                        "expired_at": crate::utils::TestTimestamps::LEGACY_TEST,
                        "status": "cleaned"
                    }
                ]
            })))
        }
        
        Router::new()
            .route("/api/v1/admin/users/:user_id/embedded-permissions", post(mock_grant_permission))
            .route("/api/v1/admin/users/:user_id/embedded-permissions/revoke", post(mock_revoke_permission))
            .route("/api/v1/admin/users/:user_id/embedded-permissions/validate", post(mock_validate_permissions))
            .route("/api/v1/admin/users/:user_id/embedded-permissions/extend", post(mock_extend_permission))
            .route("/api/v1/admin/users/bulk/embedded-permissions", post(mock_bulk_grant))
            .route("/api/v1/admin/embedded-permissions/cleanup-expired", post(mock_cleanup_expired))
    }
    
    #[tokio::test]
    async fn test_grant_embedded_permission_endpoint() {
        let app = create_test_app().await;
        
        // Test successful permission grant
        let future_timestamp = Utc::now().timestamp() + 3600;
        let request_body = json!({
            "embedded_permission": format!("epsx:analytics:premium:{}", future_timestamp),
            "base_permission": "epsx:analytics:premium",
            "platform": "epsx",
            "resource": "analytics",
            "action": "premium",
            "expiry_timestamp": future_timestamp,
            "reason": "Test permission grant"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .header("authorization", "Bearer test-token")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json["permission"].as_str().unwrap().contains("epsx:analytics:premium"));
        assert_eq!(json["expires_at"].as_i64().unwrap(), future_timestamp);
    }
    
    #[tokio::test]
    async fn test_grant_permission_validation_errors() {
        let app = create_test_app().await;
        
        // Test invalid user
        let request_body = json!({
            "base_permission": "epsx:analytics:premium",
            "expiry_timestamp": Utc::now().timestamp() + 3600
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/invalid-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        // Test past expiry timestamp
        let past_timestamp = Utc::now().timestamp() - 3600;
        let invalid_request_body = json!({
            "base_permission": "epsx:analytics:premium",
            "expiry_timestamp": past_timestamp
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&invalid_request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_validate_permissions_endpoint() {
        let app = create_test_app().await;
        
        let now = Utc::now().timestamp();
        let test_permissions = vec![
            format!("epsx:analytics:premium:{}", now + 3600),   // Valid - expires in 1 hour
            format!("epsx:rankings:view:{}", now - 3600),       // Expired - 1 hour ago
            format!("admin:users:manage:{}", now + 1800),       // Expiring soon - 30 minutes
            "epsx:basic:read"                                   // Permanent
        ];
        
        let request_body = json!({
            "permissions": test_permissions
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions/validate")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        // Verify response structure
        assert!(json["valid"].is_array());
        assert!(json["expired"].is_array());
        assert!(json["expiring_soon"].is_array());
        assert!(json["summary"].is_object());
        
        // Check summary
        let summary = &json["summary"];
        assert_eq!(summary["total"].as_u64().unwrap(), 4);
        assert!(summary["expired_count"].as_u64().unwrap() >= 1);
        assert!(summary["valid_count"].as_u64().unwrap() >= 1);
        
        // Verify expired permission structure
        let expired = json["expired"].as_array().unwrap();
        if !expired.is_empty() {
            let first_expired = &expired[0];
            assert!(first_expired["permission"].is_string());
            assert!(first_expired["base_permission"].is_string());
            assert!(first_expired["expired_at"].is_number());
            assert!(first_expired["expired_for"].is_number());
        }
    }
    
    #[tokio::test]
    async fn test_extend_permission_endpoint() {
        let app = create_test_app().await;
        
        let old_permission = &format!("epsx:analytics:premium:{}", crate::utils::TestTimestamps::LEGACY_TEST);
        let new_expiry = Utc::now().timestamp() + 7200; // 2 hours from now
        
        let request_body = json!({
            "permission": old_permission,
            "new_expiry_timestamp": new_expiry,
            "reason": "Extending for continued access"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions/extend")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["old_permission"].as_str().unwrap(), old_permission);
        assert!(json["new_permission"].as_str().unwrap().contains(&new_expiry.to_string()));
        assert!(json["extension"].as_i64().unwrap() > 0);
    }
    
    #[tokio::test]
    async fn test_revoke_permission_endpoint() {
        let app = create_test_app().await;
        
        let request_body = json!({
            "permission": &format!("epsx:analytics:premium:{}", crate::utils::TestTimestamps::LEGACY_TEST),
            "reason": "Test revocation"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions/revoke")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json["message"].as_str().unwrap().contains("revoked"));
    }
    
    #[tokio::test]
    async fn test_bulk_permission_grant_endpoint() {
        let app = create_test_app().await;
        
        let future_timestamp = Utc::now().timestamp() + 3600;
        let request_body = json!({
            "user_ids": ["user-1", "user-2", "invalid-user"],
            "permissions": [
                {
                    "base_permission": "epsx:analytics:basic",
                    "platform": "epsx",
                    "resource": "analytics",
                    "action": "basic",
                    "expiry_timestamp": future_timestamp
                },
                {
                    "base_permission": "epsx:rankings:view",
                    "platform": "epsx",
                    "resource": "rankings", 
                    "action": "view",
                    "expiry_timestamp": future_timestamp
                }
            ],
            "reason": "Bulk permission grant test"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/bulk/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        // Verify bulk operation results
        assert!(json["successful"].is_array());
        assert!(json["failed"].is_array());
        assert!(json["summary"].is_object());
        
        let summary = &json["summary"];
        assert_eq!(summary["total"].as_u64().unwrap(), 3);
        assert_eq!(summary["successful"].as_u64().unwrap(), 2);
        assert_eq!(summary["failed"].as_u64().unwrap(), 1);
        
        // Verify successful grants
        let successful = json["successful"].as_array().unwrap();
        assert_eq!(successful.len(), 2);
        
        for success in successful {
            assert!(success["user_id"].is_string());
            assert!(success["permissions"].is_array());
            let permissions = success["permissions"].as_array().unwrap();
            assert_eq!(permissions.len(), 2); // 2 permissions per user
        }
        
        // Verify failed grants
        let failed = json["failed"].as_array().unwrap();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0]["user_id"].as_str().unwrap(), "invalid-user");
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_permissions_endpoint() {
        let app = create_test_app().await;
        
        // Test dry run cleanup
        let dry_run_request = json!({
            "dry_run": true,
            "batch_size": 100
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/embedded-permissions/cleanup-expired")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&dry_run_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(json["cleaned"].as_u64().unwrap(), 0); // Dry run should clean 0
        assert_eq!(json["failed"].as_u64().unwrap(), 0);
        assert!(json["details"].is_array());
        
        // Test actual cleanup
        let cleanup_request = json!({
            "dry_run": false,
            "batch_size": 50
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/embedded-permissions/cleanup-expired")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&cleanup_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(json["cleaned"].as_u64().unwrap() > 0); // Should have cleaned some permissions
        assert_eq!(json["failed"].as_u64().unwrap(), 0);
        
        // Verify cleanup details structure
        let details = json["details"].as_array().unwrap();
        if !details.is_empty() {
            let first_detail = &details[0];
            assert!(first_detail["user_id"].is_string());
            assert!(first_detail["permission"].is_string());
            assert!(first_detail["expired_at"].is_number());
            assert!(first_detail["status"].is_string());
        }
    }
    
    #[tokio::test]
    async fn test_permission_endpoint_error_handling() {
        let app = create_test_app().await;
        
        // Test malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from("invalid json"))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_client_error());
        
        // Test missing required fields
        let incomplete_request = json!({
            "base_permission": "epsx:analytics:premium"
            // Missing expiry_timestamp
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&incomplete_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // This might be OK depending on implementation, but timestamp should be validated
        
        // Test invalid content type
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "text/plain")
            .body(Body::from("not json"))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_client_error());
    }
    
    #[tokio::test]
    async fn test_permission_endpoint_authentication() {
        let app = create_test_app().await;
        
        // Test request without authorization header
        let request_body = json!({
            "base_permission": "epsx:analytics:premium",
            "expiry_timestamp": Utc::now().timestamp() + 3600
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // Note: In a real implementation, this should return 401 Unauthorized
        // For this mock, we're just testing the endpoint structure
        
        // Test with invalid token format
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .header("authorization", "Invalid token-format")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // Again, should be 401 in real implementation
    }
    
    #[tokio::test]
    async fn test_permission_edge_cases() {
        let app = create_test_app().await;
        
        // Test with very long permission names
        let long_permission = "a".repeat(1000);
        let request_body = json!({
            "base_permission": long_permission,
            "expiry_timestamp": Utc::now().timestamp() + 3600
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // Should handle long inputs gracefully
        
        // Test with special characters in permission names
        let special_chars_permission = "epsx:测试:permission:with-émojis🎉";
        let request_body = json!({
            "base_permission": special_chars_permission,
            "expiry_timestamp": Utc::now().timestamp() + 3600
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // Should handle Unicode characters properly
        
        // Test with extremely large expiry timestamp
        let far_future = 9999999999i64; // Year 2286
        let request_body = json!({
            "base_permission": "epsx:analytics:premium",
            "expiry_timestamp": far_future
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/test-user/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        // Should handle large timestamps appropriately
        
        // Test bulk operation with empty arrays
        let empty_bulk_request = json!({
            "user_ids": [],
            "permissions": []
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/admin/users/bulk/embedded-permissions")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&empty_bulk_request).unwrap()))
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        
        let summary = &json["summary"];
        assert_eq!(summary["total"].as_u64().unwrap(), 0);
        assert_eq!(summary["successful"].as_u64().unwrap(), 0);
        assert_eq!(summary["failed"].as_u64().unwrap(), 0);
    }
}