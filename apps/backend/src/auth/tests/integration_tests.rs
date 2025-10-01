// Integration tests for the complete centralized permission system
// Tests the entire system working together: Authority + Registry + Route Protection + Middleware

use crate::auth::{
    CentralizedPermissionAuthority, DatabasePermissionRegistry, PermissionGuard, PermissionState,
    PermissionValidator, RoutePermissionResolver, ValidationContext, HandlerPermissionExt,
    create_permission_authority, create_permission_registry,
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// INTEGRATION TEST UTILITIES
// ============================================================================

async fn setup_integrated_system() -> IntegratedTestSystem {
    let db_pool = get_test_db_pool().await;
    
    // Create centralized services
    let authority = Arc::new(create_permission_authority(db_pool.clone()));
    let registry = Arc::new(create_permission_registry(db_pool.clone()));
    
    // Initialize registry
    registry.initialize().await.expect("Failed to initialize registry");
    
    // Create permission guard
    let guard = PermissionGuard::new(authority.clone(), registry.clone());
    
    // Create permission state
    let permission_state = Arc::new(PermissionState::new(authority.clone(), registry.clone()));
    
    IntegratedTestSystem {
        db_pool,
        authority,
        registry,
        guard,
        permission_state,
    }
}

struct IntegratedTestSystem {
    db_pool: PgPool,
    authority: Arc<CentralizedPermissionAuthority>,
    registry: Arc<DatabasePermissionRegistry>,
    guard: PermissionGuard,
    permission_state: Arc<PermissionState>,
}

async fn get_test_db_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_wallet_permissions(db_pool: &PgPool, wallet_address: &str, permissions: &[&str]) {
    // Create test permission group
    let group_id = Uuid::new_v4();
    let permissions_array: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    
    let _ = sqlx::query!(
        r#"
        INSERT INTO permission_groups (id, name, slug, description, permissions, is_active)
        VALUES ($1, $2, $3, $4, $5, true)
        ON CONFLICT (id) DO UPDATE SET
            permissions = EXCLUDED.permissions,
            updated_at = NOW()
        "#,
        group_id,
        "Integration Test Group",
        "integration-test-group",
        "Group for integration testing",
        &permissions_array
    )
    .execute(db_pool)
    .await
    .expect("Failed to create test group");

    // Assign wallet to group
    let _ = sqlx::query!(
        r#"
        INSERT INTO wallet_group_memberships (wallet_address, group_id, is_active)
        VALUES ($1, $2, true)
        ON CONFLICT (wallet_address, group_id) DO UPDATE SET
            is_active = true,
            updated_at = NOW()
        "#,
        wallet_address,
        group_id
    )
    .execute(db_pool)
    .await
    .expect("Failed to assign wallet to test group");
}

async fn setup_test_route_permission(
    db_pool: &PgPool,
    route_pattern: &str,
    method: &str,
    permission: &str,
    priority: i32,
) {
    let _ = sqlx::query!(
        r#"
        INSERT INTO route_permissions 
            (route_pattern, http_method, required_permission, priority, is_active, description)
        VALUES ($1, $2, $3, $4, true, $5)
        ON CONFLICT (route_pattern, http_method) DO UPDATE SET
            required_permission = EXCLUDED.required_permission,
            priority = EXCLUDED.priority
        "#,
        route_pattern,
        method.to_uppercase(),
        permission,
        priority,
        "Integration test route"
    )
    .execute(db_pool)
    .await
    .expect("Failed to setup test route permission");
}

async fn cleanup_integration_test_data(db_pool: &PgPool) {
    // Clean up test data
    let _ = sqlx::query!(
        "DELETE FROM wallet_group_memberships WHERE group_id IN (SELECT id FROM permission_groups WHERE name = 'Integration Test Group')"
    )
    .execute(db_pool)
    .await;
    
    let _ = sqlx::query!(
        "DELETE FROM permission_groups WHERE name = 'Integration Test Group'"
    )
    .execute(db_pool)
    .await;
    
    let _ = sqlx::query!(
        "DELETE FROM route_permissions WHERE description = 'Integration test route'"
    )
    .execute(db_pool)
    .await;
}

fn create_test_headers(wallet_address: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-wallet-address", wallet_address.parse().unwrap());
    headers.insert("user-agent", "integration-test-client".parse().unwrap());
    headers.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
    headers
}

// ============================================================================
// END-TO-END INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_complete_permission_validation_flow() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let route_path = "/api/v1/admin/users";
    let required_permission = "admin:users:manage";
    
    // Setup test data
    setup_test_wallet_permissions(&system.db_pool, wallet, &[required_permission]).await;
    setup_test_route_permission(&system.db_pool, route_path, "GET", required_permission, 100).await;
    
    // Refresh registry to pick up route
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Create test headers
    let headers = create_test_headers(wallet);
    
    // Test complete flow: route resolution -> permission validation
    let route_validation = system.guard
        .validate_route(wallet, "GET", route_path, &headers)
        .await;
    
    assert!(route_validation.is_ok());
    let validation = route_validation.unwrap();
    assert!(validation.granted, "Permission should be granted for complete flow");
    assert_eq!(validation.required_permission, Some(required_permission.to_string()));
    assert!(!validation.is_public_route);
    
    // Verify individual components work
    
    // 1. Route resolution
    let resolved_permission = system.registry
        .resolve_route_permission("GET", route_path)
        .await;
    assert!(resolved_permission.is_ok());
    assert_eq!(resolved_permission.unwrap(), Some(required_permission.to_string()));
    
    // 2. Permission validation
    let context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("integration-test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::Utc::now(),
        route_path: route_path.to_string(),
        http_method: "GET".to_string(),
    };
    
    let permission_result = system.authority
        .validate_permission(wallet, required_permission, &context)
        .await;
    
    assert!(permission_result.is_ok());
    let perm_validation = permission_result.unwrap();
    assert!(perm_validation.granted);
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_permission_denied_flow() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let route_path = "/api/v1/admin/users";
    let required_permission = "admin:users:manage";
    let granted_permission = "epsx:analytics:read"; // Different permission
    
    // Setup test data - grant different permission than required
    setup_test_wallet_permissions(&system.db_pool, wallet, &[granted_permission]).await;
    setup_test_route_permission(&system.db_pool, route_path, "GET", required_permission, 100).await;
    
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let headers = create_test_headers(wallet);
    
    // Test permission denied flow
    let route_validation = system.guard
        .validate_route(wallet, "GET", route_path, &headers)
        .await;
    
    assert!(route_validation.is_ok());
    let validation = route_validation.unwrap();
    assert!(!validation.granted, "Permission should be denied");
    assert_eq!(validation.required_permission, Some(required_permission.to_string()));
    
    // Verify the reason for denial
    if let Some(result) = validation.validation_result {
        assert!(!result.granted);
        assert!(result.reason.is_some());
    }
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_public_route_handling() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let public_route = "/api/auth/web3/challenge";
    
    // Public routes should not require permissions
    let headers = create_test_headers(wallet);
    
    let route_validation = system.guard
        .validate_route(wallet, "POST", public_route, &headers)
        .await;
    
    assert!(route_validation.is_ok());
    let validation = route_validation.unwrap();
    
    // Public routes should be granted regardless of permissions
    assert!(validation.granted || validation.is_public_route, 
        "Public route should be accessible");
    
    // Test route resolution for public route
    let resolved = system.registry
        .resolve_route_permission("POST", public_route)
        .await;
    
    assert!(resolved.is_ok());
    if let Some(permission) = resolved.unwrap() {
        assert_eq!(permission, "public");
    }
}

#[tokio::test]
async fn test_wildcard_permission_integration() {
    let system = setup_integrated_system().await;
    let admin_wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Grant wildcard admin permission
    setup_test_wallet_permissions(&system.db_pool, admin_wallet, &["admin:*:*"]).await;
    
    // Setup various admin routes
    let admin_routes = [
        ("/api/admin/users", "admin:users:read"),
        ("/api/admin/permission-groups", "admin:permission-groups:manage"),
        ("/api/admin/analytics", "admin:analytics:view"),
    ];
    
    for (route, permission) in &admin_routes {
        setup_test_route_permission(&system.db_pool, route, "GET", permission, 100).await;
    }
    
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let headers = create_test_headers(admin_wallet);
    
    // Test that wildcard permission grants access to all admin routes
    for (route, _) in &admin_routes {
        let validation = system.guard
            .validate_route(admin_wallet, "GET", route, &headers)
            .await;
        
        assert!(validation.is_ok(), "Failed to validate route: {}", route);
        let result = validation.unwrap();
        assert!(result.granted, "Admin should have access to route: {}", route);
    }
    
    // Test that wildcard doesn't grant non-admin permissions
    setup_test_route_permission(&system.db_pool, "/api/v1/analytics", "GET", "epsx:analytics:read", 100).await;
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let non_admin_validation = system.guard
        .validate_route(admin_wallet, "GET", "/api/v1/analytics", &headers)
        .await;
    
    assert!(non_admin_validation.is_ok());
    let result = non_admin_validation.unwrap();
    assert!(!result.granted, "Admin wildcard should not grant non-admin permissions");
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_bulk_validation_integration() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup mixed permissions
    let granted_permissions = [
        "epsx:analytics:read",
        "epsx:data:access",
        "admin:users:read",
    ];
    
    setup_test_wallet_permissions(&system.db_pool, wallet, &granted_permissions).await;
    
    // Test permissions to validate
    let test_permissions = vec![
        "epsx:analytics:read".to_string(),    // Should be granted
        "epsx:data:access".to_string(),       // Should be granted
        "admin:users:read".to_string(),       // Should be granted
        "admin:users:write".to_string(),      // Should be denied
        "epsx:premium:features".to_string(),  // Should be denied
    ];
    
    let context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("integration-test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::Utc::now(),
        route_path: "/integration/test".to_string(),
        http_method: "POST".to_string(),
    };
    
    // Perform bulk validation
    let bulk_result = system.authority
        .bulk_validate_permissions(wallet, &test_permissions, &context)
        .await;
    
    assert!(bulk_result.is_ok());
    let result = bulk_result.unwrap();
    
    assert_eq!(result.total_permissions, 5);
    assert_eq!(result.granted_count, 3);
    assert_eq!(result.denied_count, 2);
    assert!(result.validation_time_ms > 0);
    
    // Verify individual results
    assert!(result.results.get("epsx:analytics:read").unwrap().granted);
    assert!(result.results.get("epsx:data:access").unwrap().granted);
    assert!(result.results.get("admin:users:read").unwrap().granted);
    assert!(!result.results.get("admin:users:write").unwrap().granted);
    assert!(!result.results.get("epsx:premium:features").unwrap().granted);
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_caching_across_components() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let route_path = "/api/test/cached";
    let permission = "test:cache:read";
    
    // Setup test data
    setup_test_wallet_permissions(&system.db_pool, wallet, &[permission]).await;
    setup_test_route_permission(&system.db_pool, route_path, "GET", permission, 100).await;
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let headers = create_test_headers(wallet);
    
    // First validation - should populate both route and permission caches
    let start_time = std::time::Instant::now();
    let validation1 = system.guard
        .validate_route(wallet, "GET", route_path, &headers)
        .await;
    let first_elapsed = start_time.elapsed();
    
    assert!(validation1.is_ok());
    assert!(validation1.unwrap().granted);
    
    // Second validation - should hit both caches
    let start_time = std::time::Instant::now();
    let validation2 = system.guard
        .validate_route(wallet, "GET", route_path, &headers)
        .await;
    let second_elapsed = start_time.elapsed();
    
    assert!(validation2.is_ok());
    assert!(validation2.unwrap().granted);
    
    // Cache hits should be faster
    assert!(second_elapsed < first_elapsed, 
        "Cached validation ({:?}) should be faster than initial ({:?})", 
        second_elapsed, first_elapsed);
    
    // Verify cache statistics
    let authority_stats = system.authority.get_cache_stats().await;
    let registry_stats = system.registry.get_cache_stats().await;
    
    assert!(authority_stats.permission_hits > 0 || authority_stats.permission_misses > 0);
    assert!(registry_stats.hits > 0 || registry_stats.misses > 0);
    
    println!("Authority cache: {}% hit rate", 
        if authority_stats.permission_hits + authority_stats.permission_misses > 0 {
            authority_stats.permission_hits * 100 / (authority_stats.permission_hits + authority_stats.permission_misses)
        } else { 0 });
    
    println!("Registry cache: {}% hit rate",
        if registry_stats.hits + registry_stats.misses > 0 {
            registry_stats.hits * 100 / (registry_stats.hits + registry_stats.misses)
        } else { 0 });
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_route_priority_with_permissions() {
    let system = setup_integrated_system().await;
    let admin_wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let regular_wallet = "0x1234567890123456789012345678901234567890";
    
    // Setup admin with admin permissions
    setup_test_wallet_permissions(&system.db_pool, admin_wallet, &["admin:*:*"]).await;
    // Setup regular user with basic permissions
    setup_test_wallet_permissions(&system.db_pool, regular_wallet, &["epsx:data:read"]).await;
    
    // Setup overlapping routes with different priorities
    setup_test_route_permission(&system.db_pool, "/api/test/priority", "GET", "epsx:data:read", 100).await;
    setup_test_route_permission(&system.db_pool, "/api/test/**", "GET", "admin:api:access", 900).await;
    
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    let admin_headers = create_test_headers(admin_wallet);
    let regular_headers = create_test_headers(regular_wallet);
    
    // Test that admin gets higher priority route (admin:api:access)
    let admin_validation = system.guard
        .validate_route(admin_wallet, "GET", "/api/test/priority", &admin_headers)
        .await;
    
    assert!(admin_validation.is_ok());
    let admin_result = admin_validation.unwrap();
    assert!(admin_result.granted, "Admin should be granted access");
    assert_eq!(admin_result.required_permission, Some("admin:api:access".to_string()));
    
    // Test that regular user gets same high-priority route but is denied
    let regular_validation = system.guard
        .validate_route(regular_wallet, "GET", "/api/test/priority", &regular_headers)
        .await;
    
    assert!(regular_validation.is_ok());
    let regular_result = regular_validation.unwrap();
    assert!(!regular_result.granted, "Regular user should be denied admin route");
    assert_eq!(regular_result.required_permission, Some("admin:api:access".to_string()));
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

#[tokio::test]
async fn test_permission_state_convenience_methods() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "epsx:analytics:read";
    
    setup_test_wallet_permissions(&system.db_pool, wallet, &[permission]).await;
    
    // Test convenience method for permission validation
    let has_permission = system.permission_state
        .validate_permission(permission, wallet)
        .await;
    
    assert!(has_permission.is_ok());
    assert!(has_permission.unwrap(), "Wallet should have permission");
    
    // Test require_permission method
    let require_result = system.permission_state
        .require_permission(permission, wallet)
        .await;
    
    assert!(require_result.is_ok(), "Required permission should be granted");
    
    // Test with denied permission
    let denied_permission = "admin:users:manage";
    let denied_result = system.permission_state
        .require_permission(denied_permission, wallet)
        .await;
    
    assert!(denied_result.is_err(), "Denied permission should return error");
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

// ============================================================================
// PERFORMANCE INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_system_performance_under_load() {
    let system = setup_integrated_system().await;
    let num_wallets = 20;
    let num_routes = 50;
    
    // Setup multiple wallets with various permissions
    let mut test_wallets = Vec::new();
    for i in 0..num_wallets {
        let wallet = format!("0x{:040x}", i);
        test_wallets.push(wallet.clone());
        
        let permissions = if i % 3 == 0 {
            vec!["admin:*:*"]
        } else if i % 2 == 0 {
            vec!["epsx:analytics:read", "epsx:data:access"]
        } else {
            vec!["epsx:basic:access"]
        };
        
        setup_test_wallet_permissions(&system.db_pool, &wallet, &permissions).await;
    }
    
    // Setup multiple routes
    for i in 0..num_routes {
        let route = format!("/api/perf/test/{}", i);
        let permission = if i % 4 == 0 {
            "admin:perf:access"
        } else if i % 2 == 0 {
            "epsx:analytics:read"
        } else {
            "epsx:basic:access"
        };
        
        setup_test_route_permission(&system.db_pool, &route, "GET", permission, 100 - i).await;
    }
    
    system.registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Perform load test
    let start_time = std::time::Instant::now();
    let mut total_validations = 0;
    let mut successful_validations = 0;
    
    for wallet in &test_wallets {
        let headers = create_test_headers(wallet);
        
        for route_idx in 0..num_routes {
            let route = format!("/api/perf/test/{}", route_idx);
            
            let validation = system.guard
                .validate_route(wallet, "GET", &route, &headers)
                .await;
            
            total_validations += 1;
            
            if let Ok(result) = validation {
                if result.granted {
                    successful_validations += 1;
                }
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    let validations_per_second = (total_validations as f64) / elapsed.as_secs_f64();
    
    println!("Performance test results:");
    println!("  Total validations: {}", total_validations);
    println!("  Successful validations: {}", successful_validations);
    println!("  Time elapsed: {:?}", elapsed);
    println!("  Validations per second: {:.2}", validations_per_second);
    
    // Performance assertions
    assert!(validations_per_second > 100.0, 
        "System should handle at least 100 validations per second, got: {:.2}", 
        validations_per_second);
    
    assert!(elapsed.as_millis() < 10000, 
        "Load test took too long: {}ms", elapsed.as_millis());
    
    // Check cache effectiveness
    let authority_stats = system.authority.get_cache_stats().await;
    let registry_stats = system.registry.get_cache_stats().await;
    
    // Should have significant cache hits after initial misses
    let authority_hit_rate = if authority_stats.permission_hits + authority_stats.permission_misses > 0 {
        authority_stats.permission_hits as f64 / 
        (authority_stats.permission_hits + authority_stats.permission_misses) as f64
    } else { 0.0 };
    
    let registry_hit_rate = if registry_stats.hits + registry_stats.misses > 0 {
        registry_stats.hits as f64 / (registry_stats.hits + registry_stats.misses) as f64
    } else { 0.0 };
    
    println!("Cache performance:");
    println!("  Authority hit rate: {:.2}%", authority_hit_rate * 100.0);
    println!("  Registry hit rate: {:.2}%", registry_hit_rate * 100.0);
    
    // Expect reasonable cache hit rates under load
    assert!(authority_hit_rate > 0.5, 
        "Authority cache hit rate should be > 50%, got: {:.2}%", authority_hit_rate * 100.0);
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}

// ============================================================================
// ERROR HANDLING INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_system_resilience_to_database_issues() {
    let system = setup_integrated_system().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup initial permissions
    setup_test_wallet_permissions(&system.db_pool, wallet, &["epsx:analytics:read"]).await;
    
    // First validation to populate cache
    let headers = create_test_headers(wallet);
    let context = ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("resilience-test".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::Utc::now(),
        route_path: "/test/resilience".to_string(),
        http_method: "GET".to_string(),
    };
    
    let validation1 = system.authority
        .validate_permission(wallet, "epsx:analytics:read", &context)
        .await;
    
    assert!(validation1.is_ok());
    assert!(validation1.unwrap().granted);
    
    // Test that system handles database errors gracefully
    // (In a real test, you might temporarily break the connection or use a mock)
    
    // For now, test invalid wallet addresses
    let invalid_wallet = "invalid-wallet-address";
    let invalid_validation = system.authority
        .validate_permission(invalid_wallet, "epsx:analytics:read", &context)
        .await;
    
    // Should handle gracefully without crashing
    match invalid_validation {
        Ok(result) => assert!(!result.granted),
        Err(_) => {}, // Error is acceptable for invalid input
    }
    
    // Valid wallet should still work (cache should help)
    let validation2 = system.authority
        .validate_permission(wallet, "epsx:analytics:read", &context)
        .await;
    
    assert!(validation2.is_ok());
    
    // Cleanup
    cleanup_integration_test_data(&system.db_pool).await;
}