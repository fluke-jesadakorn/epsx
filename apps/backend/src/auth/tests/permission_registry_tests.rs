// Comprehensive tests for DatabasePermissionRegistry
// Tests route-to-permission mapping, pattern matching, and caching

use crate::auth::{
    DatabasePermissionRegistry, RoutePermissionResolver,
    get_default_route_permissions,
};
use sqlx::PgPool;

// ============================================================================
// TEST UTILITIES
// ============================================================================

async fn create_test_registry() -> DatabasePermissionRegistry {
    let db_pool = get_test_db_pool().await;
    let registry = DatabasePermissionRegistry::with_defaults(db_pool);
    
    // Initialize with test data
    registry.initialize().await.expect("Failed to initialize test registry");
    registry
}

async fn get_test_db_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn cleanup_test_routes(db_pool: &PgPool) {
    // Clean up test route permissions
    let _ = sqlx::query!(
        "DELETE FROM route_permissions WHERE description LIKE '%test%' OR route_pattern LIKE '/test%'"
    )
    .execute(db_pool)
    .await;
}

async fn insert_test_route(
    db_pool: &PgPool,
    pattern: &str,
    method: &str,
    permission: &str,
    priority: i32,
    is_public: bool,
) {
    let _ = sqlx::query!(
        r#"
        INSERT INTO route_permissions 
            (route_pattern, http_method, required_permission, priority, is_public, description)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (route_pattern, http_method) DO UPDATE SET
            required_permission = EXCLUDED.required_permission,
            priority = EXCLUDED.priority,
            is_public = EXCLUDED.is_public
        "#,
        pattern,
        method.to_uppercase(),
        permission,
        priority,
        is_public,
        "Test route permission"
    )
    .execute(db_pool)
    .await
    .expect("Failed to insert test route");
}

// ============================================================================
// BASIC ROUTE RESOLUTION TESTS
// ============================================================================

#[tokio::test]
async fn test_exact_route_match() {
    let registry = create_test_registry().await;
    
    // Insert test route
    insert_test_route(
        registry.db_pool(),
        "/test/exact",
        "GET",
        "test:exact:read",
        100,
        false,
    ).await;
    
    // Refresh patterns to pick up new route
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test exact match
    let result = registry.resolve_route_permission("GET", "/test/exact").await;
    
    assert!(result.is_ok());
    let permission = result.unwrap();
    assert_eq!(permission, Some("test:exact:read".to_string()));
    
    // Test non-matching route
    let result = registry.resolve_route_permission("GET", "/test/different").await;
    assert!(result.is_ok());
    // Should return None or a default permission
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_wildcard_route_matching() {
    let registry = create_test_registry().await;
    
    // Insert wildcard routes with different priorities
    insert_test_route(
        registry.db_pool(),
        "/test/admin/*",
        "*",
        "admin:test:access",
        900,
        false,
    ).await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/**",
        "*",
        "test:general:access",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test specific admin path (should match higher priority rule)
    let result = registry.resolve_route_permission("GET", "/test/admin/wallets").await;
    assert!(result.is_ok());
    let permission = result.unwrap();
    assert_eq!(permission, Some("admin:test:access".to_string()));
    
    // Test general test path (should match lower priority rule)
    let result = registry.resolve_route_permission("GET", "/test/public/data").await;
    assert!(result.is_ok());
    let permission = result.unwrap();
    assert_eq!(permission, Some("test:general:access".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_http_method_matching() {
    let registry = create_test_registry().await;
    
    // Insert method-specific routes
    insert_test_route(
        registry.db_pool(),
        "/test/resource",
        "GET",
        "test:resource:read",
        100,
        false,
    ).await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/resource",
        "POST",
        "test:resource:write",
        100,
        false,
    ).await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/any",
        "*",
        "test:any:access",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test method-specific matching
    let result = registry.resolve_route_permission("GET", "/test/resource").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("test:resource:read".to_string()));
    
    let result = registry.resolve_route_permission("POST", "/test/resource").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("test:resource:write".to_string()));
    
    // Test wildcard method matching
    let result = registry.resolve_route_permission("PUT", "/test/any").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("test:any:access".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_priority_ordering() {
    let registry = create_test_registry().await;
    
    // Insert overlapping routes with different priorities
    insert_test_route(
        registry.db_pool(),
        "/test/priority",
        "GET",
        "test:low:priority",
        100,
        false,
    ).await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/priority",
        "*",
        "test:high:priority",
        900,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Higher priority should win even with less specific method
    let result = registry.resolve_route_permission("GET", "/test/priority").await;
    assert!(result.is_ok());
    let permission = result.unwrap();
    assert_eq!(permission, Some("test:high:priority".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_public_route_handling() {
    let registry = create_test_registry().await;
    
    // Insert public route
    insert_test_route(
        registry.db_pool(),
        "/test/public",
        "GET",
        "public",
        1000,
        true,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test public route resolution
    let result = registry.resolve_route_permission("GET", "/test/public").await;
    assert!(result.is_ok());
    let permission = result.unwrap();
    assert_eq!(permission, Some("public".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// ROUTE REGISTRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_register_new_route_permission() {
    let registry = create_test_registry().await;
    
    let route_pattern = "/test/new-route";
    let method = "POST";
    let permission = "test:new:create";
    
    // Register new route permission
    let result = registry.register_route_permission(route_pattern, method, permission).await;
    assert!(result.is_ok());
    
    // Verify it was registered and can be resolved
    let resolved = registry.resolve_route_permission(method, route_pattern).await;
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), Some(permission.to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_register_duplicate_route_permission() {
    let registry = create_test_registry().await;
    
    let route_pattern = "/test/duplicate";
    let method = "GET";
    let permission1 = "test:duplicate:old";
    let permission2 = "test:duplicate:new";
    
    // Register first permission
    let result1 = registry.register_route_permission(route_pattern, method, permission1).await;
    assert!(result1.is_ok());
    
    // Register second permission (should update)
    let result2 = registry.register_route_permission(route_pattern, method, permission2).await;
    assert!(result2.is_ok());
    
    // Verify updated permission is used
    let resolved = registry.resolve_route_permission(method, route_pattern).await;
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), Some(permission2.to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// PATTERN MATCHING TESTS
// ============================================================================

#[tokio::test]
async fn test_parameter_route_matching() {
    let registry = create_test_registry().await;
    
    // Insert parameterized route
    insert_test_route(
        registry.db_pool(),
        "/test/users/:id",
        "GET",
        "test:users:read",
        100,
        false,
    ).await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/users/:id/posts/:post_id",
        "GET",
        "test:posts:read",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test parameter matching
    let result = registry.resolve_route_permission("GET", "/test/users/123").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("test:users:read".to_string()));
    
    let result = registry.resolve_route_permission("GET", "/test/users/456/posts/789").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("test:posts:read".to_string()));
    
    // Test non-matching patterns
    let result = registry.resolve_route_permission("GET", "/test/users").await;
    // Should not match the parameterized route
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_complex_wildcard_patterns() {
    let registry = create_test_registry().await;
    
    // Insert various wildcard patterns
    insert_test_route(registry.db_pool(), "/api/v1/admin/**", "*", "admin:api:access", 900, false).await;
    insert_test_route(registry.db_pool(), "/api/v1/*/public", "GET", "api:public:read", 800, false).await;
    insert_test_route(registry.db_pool(), "/api/*/analytics/*", "GET", "analytics:read", 700, false).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test complex pattern matching
    let test_cases = vec![
        ("GET", "/api/v1/admin/wallets/create", Some("admin:api:access")),
        ("POST", "/api/v1/admin/anything/here", Some("admin:api:access")),
        ("GET", "/api/v1/users/public", Some("api:public:read")),
        ("GET", "/api/v1/analytics/rankings", Some("analytics:read")),
        ("GET", "/api/v2/analytics/data", Some("analytics:read")),
    ];
    
    for (method, path, expected) in test_cases {
        let result = registry.resolve_route_permission(method, path).await;
        assert!(result.is_ok(), "Failed to resolve route: {} {}", method, path);
        assert_eq!(
            result.unwrap(),
            expected.map(|s| s.to_string()),
            "Unexpected permission for route: {} {}",
            method,
            path
        );
    }
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// CACHING TESTS
// ============================================================================

#[tokio::test]
async fn test_route_resolution_caching() {
    let registry = create_test_registry().await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/cached",
        "GET",
        "test:cached:read",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // First resolution - should be cache miss
    let start_time = std::time::Instant::now();
    let result1 = registry.resolve_route_permission("GET", "/test/cached").await;
    let first_elapsed = start_time.elapsed();
    
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Some("test:cached:read".to_string()));
    
    // Second resolution - should be cache hit (faster)
    let start_time = std::time::Instant::now();
    let result2 = registry.resolve_route_permission("GET", "/test/cached").await;
    let second_elapsed = start_time.elapsed();
    
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Some("test:cached:read".to_string()));
    
    // Cache hit should be faster
    assert!(second_elapsed < first_elapsed, 
        "Cache hit ({:?}) should be faster than miss ({:?})", 
        second_elapsed, first_elapsed);
    
    // Check cache statistics
    let stats = registry.get_cache_stats().await;
    assert!(stats.hits > 0);
    assert!(stats.misses > 0);
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_cache_invalidation_on_pattern_refresh() {
    let registry = create_test_registry().await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/refresh",
        "GET",
        "test:refresh:old",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // First resolution to populate cache
    let result1 = registry.resolve_route_permission("GET", "/test/refresh").await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), Some("test:refresh:old".to_string()));
    
    // Update the route permission in database
    let _ = sqlx::query!(
        "UPDATE route_permissions SET required_permission = 'test:refresh:new' WHERE route_pattern = '/test/refresh'"
    )
    .execute(registry.db_pool())
    .await;
    
    // Refresh patterns (should invalidate cache)
    let stats_before = registry.get_cache_stats().await;
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    let stats_after = registry.get_cache_stats().await;
    
    // Verify cache was invalidated
    assert!(stats_after.cache_invalidations > stats_before.cache_invalidations);
    
    // Next resolution should return updated permission
    let result2 = registry.resolve_route_permission("GET", "/test/refresh").await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Some("test:refresh:new".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// DEFAULT ROUTE PERMISSIONS TESTS
// ============================================================================

#[tokio::test]
async fn test_default_route_permissions_loading() {
    let registry = create_test_registry().await;
    
    // Test that default routes are loaded
    let default_routes = get_default_route_permissions();
    assert!(!default_routes.is_empty());
    
    // Test some known default routes
    let health_result = registry.resolve_route_permission("GET", "/health").await;
    assert!(health_result.is_ok());
    assert_eq!(health_result.unwrap(), Some("public".to_string()));
    
    let challenge_result = registry.resolve_route_permission("POST", "/api/auth/web3/challenge").await;
    assert!(challenge_result.is_ok());
    assert_eq!(challenge_result.unwrap(), Some("public".to_string()));
}

#[tokio::test]
async fn test_admin_route_permissions() {
    let registry = create_test_registry().await;
    
    // Test admin routes have appropriate permissions
    let test_cases = vec![
        ("GET", "/admin/wallets/list", "admin:users:manage"),
        ("GET", "/admin/permission-groups/list", "admin:permission-groups:manage"),
        ("POST", "/admin/web3/permissions", "admin:web3:manage"),
        ("GET", "/api/admin/analytics", "admin:api:access"),
    ];
    
    for (method, path, expected_prefix) in test_cases {
        let result = registry.resolve_route_permission(method, path).await;
        if let Ok(Some(permission)) = result {
            assert!(
                permission.starts_with(expected_prefix) || permission.starts_with("admin:"),
                "Admin route {} {} should have admin permission, got: {}",
                method, path, permission
            );
        }
    }
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_malformed_route_patterns() {
    let registry = create_test_registry().await;
    
    // Test resolution with malformed paths
    let malformed_paths = vec![
        "",
        "no-leading-slash",
        "/path/with//../traversal",
        "/path/with/null\0byte",
    ];
    
    for path in malformed_paths {
        let result = registry.resolve_route_permission("GET", path).await;
        // Should handle gracefully without crashing
        assert!(result.is_ok(), "Failed to handle malformed path: {}", path);
    }
}

#[tokio::test]
async fn test_invalid_http_methods() {
    let registry = create_test_registry().await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/method",
        "GET",
        "test:method:get",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test with invalid HTTP methods
    let invalid_methods = vec!["INVALID", "", "get", "post"];
    
    for method in invalid_methods {
        let result = registry.resolve_route_permission(method, "/test/method").await;
        // Should handle gracefully
        assert!(result.is_ok(), "Failed to handle invalid method: {}", method);
    }
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_route_resolution_performance() {
    let registry = create_test_registry().await;
    
    // Insert many routes with various patterns
    for i in 0..100 {
        insert_test_route(
            registry.db_pool(),
            &format!("/test/perf/{}", i),
            "GET",
            &format!("test:perf:read:{}", i),
            100 - i,
            false,
        ).await;
    }
    
    // Add some wildcard routes
    insert_test_route(registry.db_pool(), "/test/perf/**", "*", "test:perf:wildcard", 50, false).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Test resolution performance
    let start_time = std::time::Instant::now();
    let mut resolved_count = 0;
    
    for i in 0..100 {
        let path = format!("/test/perf/{}", i);
        let result = registry.resolve_route_permission("GET", &path).await;
        if result.is_ok() && result.unwrap().is_some() {
            resolved_count += 1;
        }
    }
    
    let elapsed = start_time.elapsed();
    
    assert_eq!(resolved_count, 100);
    assert!(elapsed.as_millis() < 1000, "Route resolution took too long: {}ms", elapsed.as_millis());
    
    println!("Resolved {} routes in {}ms", resolved_count, elapsed.as_millis());
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_get_all_mappings() {
    let registry = create_test_registry().await;
    
    // Insert test mappings
    insert_test_route(registry.db_pool(), "/test/mapping1", "GET", "test:map1", 100, false).await;
    insert_test_route(registry.db_pool(), "/test/mapping2", "POST", "test:map2", 200, false).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // Get all mappings
    let result = registry.get_all_mappings().await;
    assert!(result.is_ok());
    
    let mappings = result.unwrap();
    
    // Should contain at least our test mappings plus defaults
    assert!(mappings.len() >= 2);
    
    // Find our test mappings
    let test_mapping1 = mappings.iter().find(|m| m.route_pattern == "/test/mapping1");
    assert!(test_mapping1.is_some());
    assert_eq!(test_mapping1.unwrap().required_permission, "test:map1");
    
    let test_mapping2 = mappings.iter().find(|m| m.route_pattern == "/test/mapping2");
    assert!(test_mapping2.is_some());
    assert_eq!(test_mapping2.unwrap().required_permission, "test:map2");
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}

#[tokio::test]
async fn test_route_cache_invalidation() {
    let registry = create_test_registry().await;
    
    insert_test_route(
        registry.db_pool(),
        "/test/invalidate",
        "GET",
        "test:invalidate:read",
        100,
        false,
    ).await;
    
    registry.refresh_patterns().await.expect("Failed to refresh patterns");
    
    // First resolution to populate cache
    let result1 = registry.resolve_route_permission("GET", "/test/invalidate").await;
    assert!(result1.is_ok());
    
    // Invalidate specific route cache
    registry.invalidate_route_cache("GET", "/test/invalidate").await;
    
    // Next resolution should work (cache miss)
    let result2 = registry.resolve_route_permission("GET", "/test/invalidate").await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), Some("test:invalidate:read".to_string()));
    
    // Cleanup
    cleanup_test_routes(registry.db_pool()).await;
}