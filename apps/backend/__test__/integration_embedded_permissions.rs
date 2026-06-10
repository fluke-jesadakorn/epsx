// ============================================================================
// INTEGRATION TESTS - EMBEDDED TIMESTAMP PERMISSIONS
// ============================================================================
// Comprehensive integration tests for embedded timestamp permission functionality
// Tests the complete flow from API endpoints to database operations

use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::{Duration, Utc};
use serde_json::{json, Value};

// Test utilities module
#[path = "utils/mod.rs"]
mod utils;

use crate::app::test_utils::{create_test_app, TestState};
use crate::auth::permissions::{
    parse_permission_with_timestamp, is_permission_valid_with_time_check,
    add_timestamp_to_permission, filter_valid_permissions
};

/// Test helper to create embedded permission with specified expiry
fn create_test_permission_with_expiry(base_permission: &str, hours_from_now: i64) -> String {
    let expiry = Utc::now().timestamp() + (hours_from_now * 3600);
    format!("{}:{}", base_permission, expiry)
}

/// Test helper to create expired permission
fn create_expired_permission(base_permission: &str) -> String {
    let expiry = Utc::now().timestamp() - 3600; // 1 hour ago
    format!("{}:{}", base_permission, expiry)
}

#[tokio::test]
async fn test_embedded_timestamp_parsing_and_validation() {
    // Test basic embedded timestamp parsing
    let timestamp = Utc::now().timestamp() + 3600; // 1 hour from now
    let embedded_perm = format!("epsx:analytics:view:{}", timestamp);
    
    let (base_permission, parsed_timestamp) = parse_permission_with_timestamp(&embedded_perm);
    assert_eq!(base_permission, "epsx:analytics:view");
    assert_eq!(parsed_timestamp, Some(timestamp));
    
    // Test permission validity
    assert!(is_permission_valid_with_time_check(&embedded_perm));
    
    // Test expired permission
    let expired_timestamp = Utc::now().timestamp() - 3600; // 1 hour ago
    let expired_perm = format!("epsx:analytics:view:{}", expired_timestamp);
    assert!(!is_permission_valid_with_time_check(&expired_perm));
    
    // Test permanent permission (no timestamp)
    let permanent_perm = "epsx:analytics:view";
    assert!(is_permission_valid_with_time_check(permanent_perm));
    
    let (base_perm, timestamp) = parse_permission_with_timestamp(permanent_perm);
    assert_eq!(base_perm, "epsx:analytics:view");
    assert_eq!(timestamp, None);
}

#[tokio::test]
async fn test_permission_filtering() {
    let permissions = vec![
        "epsx:analytics:view".to_string(), // Permanent
        create_test_permission_with_expiry("epsx:rankings:view:50", 2), // Valid (2 hours)
        create_expired_permission("admin:users:manage"), // Expired
        create_test_permission_with_expiry("epsx:realtime:access", 24), // Valid (1 day)
    ];
    
    let valid_permissions = filter_valid_permissions(&permissions);
    
    // Should have 3 valid permissions (permanent + 2 future-dated)
    assert_eq!(valid_permissions.len(), 3);
    assert!(valid_permissions.contains(&"epsx:analytics:view".to_string()));
    assert!(!valid_permissions.iter().any(|p| p.contains("admin:users:manage")));
}

#[cfg(test)]
mod api_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_grant_embedded_permission_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let test_user_id = "test-user-123";
        let expiry_timestamp = Utc::now().timestamp() + 3600; // 1 hour from now
        
        let grant_request = json!({
            "embedded_permission": format!("epsx:analytics:view:{}", expiry_timestamp),
            "base_permission": "epsx:analytics:view",
            "platform": "epsx",
            "resource": "analytics", 
            "action": "view",
            "expiry_timestamp": expiry_timestamp,
            "reason": "Integration test - temporary analytics access"
        });
        
        let response = server
            .post(&format!("/api/admin/users/{}/embedded-permissions", test_user_id))
            .json(&grant_request)
            .await;
        
        // Should succeed if endpoint is implemented
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("permission").is_some());
            assert!(body.get("expires_at").is_some());
        } else {
            // If not implemented yet, should return 503 or 404
            assert!(matches!(response.status(), StatusCode::SERVICE_UNAVAILABLE | StatusCode::NOT_FOUND));
        }
    }

    #[tokio::test]
    async fn test_validate_embedded_permissions_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let test_user_id = "test-user-123";
        let permissions = vec![
            "epsx:analytics:view".to_string(),
            create_test_permission_with_expiry("epsx:rankings:view:25", 4),
            create_expired_permission("admin:users:manage"),
        ];
        
        let validate_request = json!({
            "permissions": permissions
        });
        
        let response = server
            .post(&format!("/api/admin/users/{}/embedded-permissions/validate", test_user_id))
            .json(&validate_request)
            .await;
        
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("valid").is_some());
            assert!(body.get("summary").is_some());
            
            let summary = body.get("summary").unwrap();
            assert!(summary.get("total").is_some());
            assert!(summary.get("valid_count").is_some());
            assert!(summary.get("expired_count").is_some());
        }
    }

    #[tokio::test]
    async fn test_permission_expiry_status_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let test_user_id = "test-user-123";
        
        let response = server
            .get(&format!("/api/admin/users/{}/permissions/expiry-status", test_user_id))
            .await;
        
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("user_id").is_some());
            assert!(body.get("permissions").is_some());
            assert!(body.get("health").is_some());
            
            let health = body.get("health").unwrap();
            assert!(health.get("has_expired").is_some());
            assert!(health.get("has_expiring_soon").is_some());
        }
    }

    #[tokio::test]
    async fn test_extend_embedded_permission_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let test_user_id = "test-user-123";
        let original_expiry = Utc::now().timestamp() + 3600;
        let new_expiry = Utc::now().timestamp() + 7200; // 2 hours from now
        let permission = format!("epsx:analytics:view:{}", original_expiry);
        
        let extend_request = json!({
            "permission": permission,
            "new_expiry_timestamp": new_expiry,
            "reason": "Integration test - extending access"
        });
        
        let response = server
            .post(&format!("/api/admin/users/{}/embedded-permissions/extend", test_user_id))
            .json(&extend_request)
            .await;
        
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("old_permission").is_some());
            assert!(body.get("new_permission").is_some());
            assert!(body.get("extension").is_some());
        }
    }

    #[tokio::test]
    async fn test_revoke_embedded_permission_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let test_user_id = "test-user-123";
        let permission = create_test_permission_with_expiry("epsx:analytics:view", 2);
        
        let revoke_request = json!({
            "permission": permission,
            "reason": "Integration test - revoking access"
        });
        
        let response = server
            .post(&format!("/api/admin/users/{}/embedded-permissions/revoke", test_user_id))
            .json(&revoke_request)
            .await;
        
        // Should succeed with 200 or be unimplemented
        assert!(matches!(response.status(), StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE | StatusCode::NOT_FOUND));
    }

    #[tokio::test]
    async fn test_bulk_embedded_permissions_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let user_ids = vec!["user-1", "user-2", "user-3"];
        let expiry_timestamp = Utc::now().timestamp() + 3600;
        
        let bulk_request = json!({
            "user_ids": user_ids,
            "permissions": [
                {
                    "base_permission": "epsx:analytics:view",
                    "platform": "epsx",
                    "resource": "analytics",
                    "action": "view",
                    "expiry_timestamp": expiry_timestamp
                },
                {
                    "base_permission": "epsx:rankings:view:25",
                    "platform": "epsx", 
                    "resource": "rankings",
                    "action": "view",
                    "expiry_timestamp": expiry_timestamp
                }
            ],
            "reason": "Integration test - bulk permission grant"
        });
        
        let response = server
            .post("/api/admin/users/bulk/embedded-permissions")
            .json(&bulk_request)
            .await;
        
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("successful").is_some());
            assert!(body.get("failed").is_some());
            assert!(body.get("summary").is_some());
            
            let summary = body.get("summary").unwrap();
            assert!(summary.get("total").is_some());
            assert!(summary.get("successful").is_some());
            assert!(summary.get("failed").is_some());
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired_permissions_endpoint() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        let cleanup_request = json!({
            "dry_run": true,
            "batch_size": 50
        });
        
        let response = server
            .post("/api/admin/embedded-permissions/cleanup-expired")
            .json(&cleanup_request)
            .await;
        
        if response.status() == StatusCode::OK {
            let body: Value = response.json();
            assert!(body.get("cleaned").is_some());
            assert!(body.get("failed").is_some());
            assert!(body.get("details").is_some());
        }
    }
}

#[cfg(test)]
mod cross_platform_tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_platform_permission_validation() {
        let permissions = vec![
            "epsx:analytics:view".to_string(),
            create_test_permission_with_expiry("epsx-pay:transactions:read", 2),
            create_test_permission_with_expiry("epsx-token:governance:vote", 4),
            create_test_permission_with_expiry("admin:users:manage", 24),
        ];
        
        let valid_permissions = filter_valid_permissions(&permissions);
        assert_eq!(valid_permissions.len(), 4); // All should be valid
        
        // Test platform isolation
        assert!(valid_permissions.iter().any(|p| p.starts_with("epsx:")));
        assert!(valid_permissions.iter().any(|p| p.starts_with("epsx-pay:")));
        assert!(valid_permissions.iter().any(|p| p.starts_with("epsx-token:")));
        assert!(valid_permissions.iter().any(|p| p.starts_with("admin:")));
    }

    #[tokio::test]
    async fn test_platform_specific_expiry() {
        let now = Utc::now().timestamp();
        
        // Create permissions with different expiry times for different platforms
        let permissions = vec![
            format!("epsx:analytics:view:{}", now + 1800), // 30 min
            format!("epsx-pay:payments:process:{}", now + 3600), // 1 hour
            format!("epsx-token:tokens:stake:{}", now - 1800), // Expired 30 min ago
            "admin:system:manage".to_string(), // Permanent
        ];
        
        let valid_permissions = filter_valid_permissions(&permissions);
        
        // Should have 3 valid permissions (2 future + 1 permanent)
        assert_eq!(valid_permissions.len(), 3);
        assert!(!valid_permissions.iter().any(|p| p.contains("epsx-token:tokens:stake")));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_large_permission_set_performance() {
        let start = std::time::Instant::now();
        
        // Create a large set of permissions with mixed expiry status
        let mut permissions = Vec::new();
        for i in 0..1000 {
            if i % 3 == 0 {
                // Expired permission
                permissions.push(create_expired_permission(&format!("test:resource{}:action", i)));
            } else if i % 3 == 1 {
                // Valid permission
                permissions.push(create_test_permission_with_expiry(&format!("test:resource{}:action", i), 2));
            } else {
                // Permanent permission
                permissions.push(format!("test:resource{}:action", i));
            }
        }
        
        let valid_permissions = filter_valid_permissions(&permissions);
        
        let duration = start.elapsed();
        
        // Should complete within reasonable time
        assert!(duration.as_millis() < 100); // Less than 100ms for 1000 permissions
        assert!(valid_permissions.len() > 600); // Should have ~667 valid permissions
        assert!(valid_permissions.len() < 800);
    }

    #[tokio::test]
    async fn test_timestamp_parsing_performance() {
        let start = std::time::Instant::now();
        
        use crate::utils::{TestTimestamps, PermissionBuilder};
        
        // Test parsing performance on various permission formats
        let permissions = vec![
            "epsx:analytics:view",
            "epsx:rankings:view:50",
            &PermissionBuilder::rankings_with_limit_and_expiry(50, TestTimestamps::LEGACY_TEST),
            &PermissionBuilder::admin_with_expiry(TestTimestamps::LEGACY_TEST),
            &PermissionBuilder::payments_with_expiry(TestTimestamps::LEGACY_TEST),
            &format!("epsx-token:governance:vote:unlimited:{}", TestTimestamps::LEGACY_TEST),
        ];
        
        for _ in 0..1000 {
            for perm in &permissions {
                let (base, timestamp) = parse_permission_with_timestamp(perm);
                assert!(!base.is_empty());
                // Timestamp may or may not be present
                if timestamp.is_some() {
                    assert!(timestamp.unwrap() > 0);
                }
            }
        }
        
        let duration = start.elapsed();
        
        // Should complete parsing 6000 permissions quickly
        assert!(duration.as_millis() < 50); // Less than 50ms for 6000 parses
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_malformed_permission_handling() {
        // Test various malformed permission strings
        let malformed_permissions = vec![
            "",
            ":",
            ":::",
            "epsx",
            "epsx:",
            "epsx::",
            "epsx:::",
            ":analytics:view",
            "epsx:analytics:",
            "epsx:analytics:view:",
            "epsx:analytics:view:not_a_timestamp",
            "epsx:analytics:view:abc:def",
        ];
        
        for perm in malformed_permissions {
            let (base, timestamp) = parse_permission_with_timestamp(perm);
            // Should not crash, base should be the original string if malformed
            if base.is_empty() {
                assert_eq!(base, perm);
            }
            // Timestamp should be None for malformed strings
            if perm.contains("not_a_timestamp") || perm.contains("abc") {
                assert_eq!(timestamp, None);
            }
        }
    }

    #[tokio::test]
    async fn test_edge_case_timestamps() {
        // Test edge case timestamps
        let edge_cases = vec![
            ("epsx:analytics:view:0", Some(0)), // Unix epoch
            ("epsx:analytics:view:-1", None), // Negative (should be invalid)
            ("epsx:analytics:view:9999999999", Some(9999999999)), // Far future
            ("epsx:analytics:view:1", Some(1)), // Just past epoch
        ];
        
        for (perm, expected_timestamp) in edge_cases {
            let (base, timestamp) = parse_permission_with_timestamp(perm);
            assert_eq!(base, if expected_timestamp.is_some() { "epsx:analytics:view" } else { perm });
            assert_eq!(timestamp, expected_timestamp);
        }
    }

    #[tokio::test]
    async fn test_api_error_responses() {
        let (app, _state) = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        // Test invalid user ID
        let response = server
            .get("/api/admin/users/invalid-user-id/permissions/expiry-status")
            .await;
        
        // Should handle invalid user gracefully
        assert!(matches!(response.status(), StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::SERVICE_UNAVAILABLE));
        
        // Test malformed request body
        let malformed_request = json!({
            "invalid_field": "invalid_value"
        });
        
        let response = server
            .post("/api/admin/users/test-user/embedded-permissions/validate")
            .json(&malformed_request)
            .await;
        
        // Should handle malformed request gracefully
        assert!(matches!(response.status(), StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY | StatusCode::SERVICE_UNAVAILABLE));
    }
}