// Tests for route protection system components
// Tests PermissionGuard, middleware builders, and handler traits

use crate::auth::{
    PermissionGuard, PermissionState, RequirePermission, HandlerPermissionExt, ValidationContext,
    create_permission_authority, create_permission_registry,
};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use chrono::Timelike;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// TEST UTILITIES
// ============================================================================

async fn setup_route_protection_test() -> (Arc<PermissionState>, PgPool) {
    let db_pool = get_test_db_pool().await;
    let authority = Arc::new(create_permission_authority(db_pool.clone()));
    let registry = Arc::new(create_permission_registry(db_pool.clone()));
    
    registry.initialize().await.expect("Failed to initialize registry");
    
    let permission_state = Arc::new(PermissionState::new(authority, registry));
    
    (permission_state, db_pool)
}

async fn get_test_db_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

fn create_test_headers(wallet_address: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-wallet-address", wallet_address.parse().unwrap());
    headers.insert("user-agent", "route-protection-test".parse().unwrap());
    headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
    headers
}

async fn setup_test_permissions(db_pool: &PgPool, wallet_address: &str, permissions: &[&str]) {
    let group_id = Uuid::new_v4();

    let _ = sqlx::query!(
        r#"
        INSERT INTO permission_groups (id, name, slug, description, is_active)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (name) DO NOTHING
        "#,
        group_id,
        "Route Protection Test Group",
        "route-protection-test",
        "Test group for route protection"
    )
    .execute(db_pool)
    .await
    .expect("Failed to create test group");

    // Create permissions and link to group
    for perm in permissions {
        let parts: Vec<&str> = perm.split(':').collect();
        if parts.len() == 3 {
            let (platform, resource, action) = (parts[0], parts[1], parts[2]);

            // Create permission
            let perm_id = sqlx::query_scalar!(
                r#"
                INSERT INTO permissions (permission_string, platform, resource, action, is_active)
                VALUES ($1, $2, $3, $4, true)
                ON CONFLICT (permission_string) DO UPDATE SET is_active = true
                RETURNING id
                "#,
                perm,
                platform,
                resource,
                action
            )
            .fetch_one(db_pool)
            .await
            .ok();

            // Link permission to group
            if let Some(pid) = perm_id {
                let _ = sqlx::query!(
                    r#"
                    INSERT INTO permission_group_memberships (group_id, permission_id)
                    VALUES ($1, $2)
                    ON CONFLICT (group_id, permission_id) DO NOTHING
                    "#,
                    group_id,
                    pid
                )
                .execute(db_pool)
                .await;
            }
        }
    }

    let _ = sqlx::query!(
        r#"
        INSERT INTO wallet_group_memberships (wallet_address, group_id, is_active)
        VALUES ($1, $2, true)
        ON CONFLICT (wallet_address, group_id) DO UPDATE SET is_active = true
        "#,
        wallet_address,
        group_id
    )
    .execute(db_pool)
    .await
    .expect("Failed to assign wallet to test group");
}

async fn cleanup_route_protection_test_data(db_pool: &PgPool) {
    let _ = sqlx::query!(
        "DELETE FROM wallet_group_memberships WHERE group_id IN (SELECT id FROM permission_groups WHERE name = 'Route Protection Test Group')"
    )
    .execute(db_pool)
    .await;
    
    let _ = sqlx::query!(
        "DELETE FROM permission_groups WHERE name = 'Route Protection Test Group'"
    )
    .execute(db_pool)
    .await;
}

// ============================================================================
// PERMISSION GUARD TESTS
// ============================================================================

#[tokio::test]
async fn test_permission_guard_basic_validation() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "test:guard:read";
    
    // Setup test permissions
    setup_test_permissions(&db_pool, wallet, &[permission]).await;
    
    // Create permission guard
    let guard = PermissionGuard::new(
        permission_state.authority.clone(),
        permission_state.registry.clone(),
    );
    
    // Test basic permission validation
    let context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::Utc::now(),
        route_path: "/test/guard".to_string(),
        http_method: "GET".to_string(),
    };
    
    let result = guard.validate(wallet, permission, context).await;
    
    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(validation_result.granted);
    assert_eq!(validation_result.permission, permission);
    
    // Test denied permission
    let denied_result = guard.has_permission(wallet, "admin:users:manage").await;
    assert!(denied_result.is_ok());
    assert!(!denied_result.unwrap());
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}

#[tokio::test]
async fn test_permission_guard_route_validation() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup permissions
    setup_test_permissions(&db_pool, wallet, &["test:route:access"]).await;
    
    // Register route permission
    let _ = sqlx::query!(
        r#"
        INSERT INTO route_permissions (route_pattern, http_method, required_permission, priority, is_active)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (route_pattern, http_method) DO UPDATE SET
            required_permission = EXCLUDED.required_permission
        "#,
        "/test/guard/route",
        "GET",
        "test:route:access",
        100
    )
    .execute(&db_pool)
    .await
    .expect("Failed to register route permission");
    
    // Refresh registry patterns
    permission_state.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let guard = PermissionGuard::new(
        permission_state.authority.clone(),
        permission_state.registry.clone(),
    );
    
    let headers = create_test_headers(wallet);
    
    // Test route validation
    let route_result = guard.validate_route(wallet, "GET", "/test/guard/route", &headers).await;
    
    assert!(route_result.is_ok());
    let validation = route_result.unwrap();
    assert!(validation.granted);
    assert_eq!(validation.required_permission, Some("test:route:access".to_string()));
    assert!(!validation.is_public_route);
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}

// ============================================================================
// PERMISSION STATE TESTS
// ============================================================================

#[tokio::test]
async fn test_permission_state_convenience_methods() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "test:state:read";
    
    setup_test_permissions(&db_pool, wallet, &[permission]).await;
    
    // Test validate_permission convenience method
    let validation_result = permission_state.validate_permission(permission, wallet).await;
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap());
    
    // Test require_permission convenience method
    let require_result = permission_state.require_permission(permission, wallet).await;
    assert!(require_result.is_ok());
    
    // Test with permission that should be denied
    let denied_require = permission_state.require_permission("admin:users:delete", wallet).await;
    assert!(denied_require.is_err());
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}

// ============================================================================
// REQUIRE PERMISSION TRAIT TESTS
// ============================================================================

#[derive(Clone)]
struct TestHandler;

#[async_trait::async_trait]
impl RequirePermission for TestHandler {
    fn required_permission() -> &'static str {
        "test:handler:access"
    }
    
    async fn custom_validation(
        &self,
        wallet_address: &str,
        context: &ValidationContext,
    ) -> Result<bool, crate::core::errors::AppError> {
        // Custom validation: only allow during business hours (for testing)
        let hour = context.timestamp.hour();
        if hour < 6 || hour > 22 {
            tracing::warn!("Access attempted outside business hours by {}", wallet_address);
            return Ok(false);
        }
        Ok(true)
    }
    
    fn permission_denied_response(&self) -> axum::response::Response {
        use axum::response::Json;
        use serde_json::json;
        
        let error_response = json!({
            "error": "permission_denied",
            "message": "Test handler access denied",
            "required_permission": Self::required_permission()
        });
        
        Json(error_response).into_response()
    }
}

#[tokio::test]
async fn test_require_permission_trait() {
    let handler = TestHandler;
    
    // Test required permission
    assert_eq!(TestHandler::required_permission(), "test:handler:access");
    
    // Test custom validation during business hours
    let business_hours_context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-15T14:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        route_path: "/test".to_string(),
        http_method: "GET".to_string(),
    };
    
    let validation_result = handler
        .custom_validation("0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695", &business_hours_context)
        .await;
    
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap());
    
    // Test custom validation outside business hours
    let after_hours_context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-15T23:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        route_path: "/test".to_string(),
        http_method: "GET".to_string(),
    };
    
    let after_hours_result = handler
        .custom_validation("0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695", &after_hours_context)
        .await;
    
    assert!(after_hours_result.is_ok());
    assert!(!after_hours_result.unwrap());
    
    // Test permission denied response
    let response = handler.permission_denied_response();
    assert_eq!(response.status(), axum::http::StatusCode::OK); // JSON response returns 200
}

// ============================================================================
// HANDLER EXTENSION TRAIT TESTS
// ============================================================================

#[tokio::test]
async fn test_handler_permission_extension() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "test:extension:read";
    
    setup_test_permissions(&db_pool, wallet, &[permission]).await;
    
    // Test extension trait methods
    let validation_result = permission_state.validate_permission(permission, wallet).await;
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap());
    
    let require_result = permission_state.require_permission(permission, wallet).await;
    assert!(require_result.is_ok());
    
    // Test denied permission
    let denied_result = permission_state.validate_permission("admin:super:admin", wallet).await;
    assert!(denied_result.is_ok());
    assert!(!denied_result.unwrap());
    
    let denied_require = permission_state.require_permission("admin:super:admin", wallet).await;
    assert!(denied_require.is_err());
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_route_protection_error_handling() {
    let (permission_state, _db_pool) = setup_route_protection_test().await;
    
    let guard = PermissionGuard::new(
        permission_state.authority.clone(),
        permission_state.registry.clone(),
    );
    
    // Test with invalid wallet address
    let invalid_wallet = "invalid-wallet-format";
    let context = ValidationContext::default();
    
    let result = guard.validate(invalid_wallet, "test:permission", context).await;
    
    // Should handle gracefully
    match result {
        Ok(validation_result) => {
            assert!(!validation_result.granted, "Invalid wallet should not be granted permissions");
        }
        Err(_) => {
            // Error is also acceptable for invalid wallet
        }
    }
    
    // Test with empty permission
    let empty_permission_result = guard.has_permission(
        "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695",
        ""
    ).await;
    
    match empty_permission_result {
        Ok(granted) => assert!(!granted, "Empty permission should not be granted"),
        Err(_) => {}, // Error is acceptable
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_route_protection_performance() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup comprehensive permissions
    let permissions = [
        "test:perf:read",
        "test:perf:write", 
        "admin:perf:access",
        "epsx:perf:analytics"
    ];
    setup_test_permissions(&db_pool, wallet, &permissions).await;
    
    let guard = PermissionGuard::new(
        permission_state.authority.clone(),
        permission_state.registry.clone(),
    );
    
    let headers = create_test_headers(wallet);
    
    // Register test routes
    for (i, permission) in permissions.iter().enumerate() {
        let route = format!("/test/perf/{}", i);
        let _ = sqlx::query!(
            r#"
            INSERT INTO route_permissions (route_pattern, http_method, required_permission, priority, is_active)
            VALUES ($1, 'GET', $2, $3, true)
            ON CONFLICT (route_pattern, http_method) DO UPDATE SET
                required_permission = EXCLUDED.required_permission
            "#,
            route,
            permission,
            100 - i as i32
        )
        .execute(&db_pool)
        .await
        .expect("Failed to register performance test route");
    }
    
    permission_state.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Performance test
    let start_time = std::time::Instant::now();
    let mut successful_validations = 0;
    
    for i in 0..100 {
        let route = format!("/test/perf/{}", i % permissions.len());
        let validation = guard.validate_route(wallet, "GET", &route, &headers).await;
        
        if let Ok(result) = validation {
            if result.granted {
                successful_validations += 1;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    let validations_per_second = 100.0 / elapsed.as_secs_f64();
    
    assert!(successful_validations > 0, "Some validations should succeed");
    assert!(elapsed.as_millis() < 5000, "Performance test took too long: {}ms", elapsed.as_millis());
    assert!(validations_per_second > 20.0, "Should handle at least 20 validations per second, got: {:.2}", validations_per_second);
    
    println!("Route protection performance: {:.2} validations/second", validations_per_second);
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}

// ============================================================================
// CONCURRENT ACCESS TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_route_validation() {
    let (permission_state, db_pool) = setup_route_protection_test().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    setup_test_permissions(&db_pool, wallet, &["test:concurrent:access"]).await;
    
    let guard = Arc::new(PermissionGuard::new(
        permission_state.authority.clone(),
        permission_state.registry.clone(),
    ));
    
    // Setup test route
    let _ = sqlx::query!(
        r#"
        INSERT INTO route_permissions (route_pattern, http_method, required_permission, priority, is_active)
        VALUES ('/test/concurrent', 'GET', 'test:concurrent:access', 100, true)
        ON CONFLICT (route_pattern, http_method) DO UPDATE SET
            required_permission = EXCLUDED.required_permission
        "#
    )
    .execute(&db_pool)
    .await
    .expect("Failed to register concurrent test route");
    
    permission_state.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Spawn concurrent validation tasks
    let num_concurrent = 10;
    let mut tasks = Vec::new();
    
    for _i in 0..num_concurrent {
        let guard_clone = guard.clone();
        let wallet_clone = wallet.to_string();
        let headers = create_test_headers(&wallet_clone);
        
        let task = tokio::spawn(async move {
            guard_clone.validate_route(&wallet_clone, "GET", "/test/concurrent", &headers).await
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks and verify results
    let mut successful_count = 0;
    
    for task in tasks {
        if let Ok(Ok(validation)) = task.await {
            if validation.granted {
                successful_count += 1;
            }
        }
    }
    
    assert_eq!(successful_count, num_concurrent, 
        "All concurrent validations should succeed");
    
    // Cleanup
    cleanup_route_protection_test_data(&db_pool).await;
}