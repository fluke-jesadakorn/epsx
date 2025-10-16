// Comprehensive tests for CentralizedPermissionAuthority
// Tests all validation, caching, and performance aspects

use crate::auth::{
    CentralizedPermissionAuthority, PermissionValidator, ValidationContext, CacheConfig,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// ============================================================================
// TEST UTILITIES
// ============================================================================

async fn create_test_authority() -> CentralizedPermissionAuthority {
    let db_pool = get_test_db_pool().await;
    CentralizedPermissionAuthority::with_defaults(db_pool)
}

async fn create_test_authority_with_cache(cache_ttl: Duration) -> CentralizedPermissionAuthority {
    let db_pool = get_test_db_pool().await;
    let cache_config = CacheConfig {
        permission_ttl: cache_ttl,
        route_mapping_ttl: cache_ttl,
        max_cache_size: 1000,
        enable_cache: true,
    };
    CentralizedPermissionAuthority::new(db_pool, Some(cache_config))
}

async fn get_test_db_pool() -> PgPool {
    // Use test database URL or create in-memory mock
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

fn create_test_context() -> ValidationContext {
    ValidationContext {
        request_id: Uuid::new_v4().to_string(),
        user_agent: Some("test-agent".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
        timestamp: chrono::Utc::now(),
        route_path: "/test".to_string(),
        http_method: "GET".to_string(),
    }
}

async fn setup_test_permissions(db_pool: &PgPool, wallet_address: &str, permissions: &[&str]) {
    // Create test permission group
    let group_id = Uuid::new_v4();

    let _ = sqlx::query!(
        r#"
        INSERT INTO permission_groups (id, name, slug, description, is_active)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (name) DO NOTHING
        "#,
        group_id,
        "Test Group",
        "test-group",
        "Test permissions group"
    )
    .execute(db_pool)
    .await;

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

    // Assign wallet to group
    let _ = sqlx::query!(
        r#"
        INSERT INTO wallet_group_memberships (wallet_address, group_id, is_active)
        VALUES ($1, $2, true)
        ON CONFLICT (wallet_address, group_id) DO NOTHING
        "#,
        wallet_address,
        group_id
    )
    .execute(db_pool)
    .await;
}

async fn cleanup_test_data(db_pool: &PgPool, wallet_address: &str) {
    // Clean up test data
    let _ = sqlx::query!(
        "DELETE FROM wallet_group_memberships WHERE wallet_address = $1",
        wallet_address
    )
    .execute(db_pool)
    .await;
    
    let _ = sqlx::query!(
        "DELETE FROM permission_groups WHERE name = 'Test Group'"
    )
    .execute(db_pool)
    .await;
}

// ============================================================================
// PERMISSION VALIDATION TESTS
// ============================================================================

#[tokio::test]
async fn test_single_permission_validation_granted() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "epsx:analytics:read";
    
    // Setup test permissions
    setup_test_permissions(&authority.db_pool(), wallet, &[permission]).await;
    
    let context = create_test_context();
    let result = authority.validate_permission(wallet, permission, &context).await;
    
    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(validation_result.granted);
    assert_eq!(validation_result.permission, permission);
    assert!(validation_result.validation_time_ms > 0);
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_single_permission_validation_denied() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let granted_permission = "epsx:analytics:read";
    let requested_permission = "admin:users:manage";
    
    // Setup permissions (grant different permission than requested)
    setup_test_permissions(&authority.db_pool(), wallet, &[granted_permission]).await;
    
    let context = create_test_context();
    let result = authority.validate_permission(wallet, requested_permission, &context).await;
    
    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(!validation_result.granted);
    assert_eq!(validation_result.permission, requested_permission);
    assert!(validation_result.reason.is_some());
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_wildcard_permission_validation() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Grant wildcard admin permission
    setup_test_permissions(&authority.db_pool(), wallet, &["admin:*:*"]).await;
    
    let context = create_test_context();
    
    // Test that specific admin permissions are granted via wildcard
    let specific_permissions = [
        "admin:users:read",
        "admin:users:write",
        "admin:permission-groups:manage",
        "admin:analytics:view",
    ];
    
    for permission in &specific_permissions {
        let result = authority.validate_permission(wallet, permission, &context).await;
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(validation_result.granted, "Permission {} should be granted via admin:*:*", permission);
    }
    
    // Test that non-admin permissions are not granted
    let result = authority.validate_permission(wallet, "epsx:analytics:read", &context).await;
    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(!validation_result.granted, "Non-admin permission should not be granted by admin:*:*");
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_bulk_permission_validation() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup mixed permissions
    let granted_perms = ["epsx:analytics:read", "epsx:data:access"];
    setup_test_permissions(&authority.db_pool(), wallet, &granted_perms).await;
    
    let context = create_test_context();
    let test_permissions = vec![
        "epsx:analytics:read".to_string(),    // Should be granted
        "epsx:data:access".to_string(),       // Should be granted
        "admin:users:manage".to_string(),     // Should be denied
        "epsx:premium:features".to_string(),  // Should be denied
    ];
    
    let result = authority.bulk_validate_permissions(wallet, &test_permissions, &context).await;
    
    assert!(result.is_ok());
    let bulk_result = result.unwrap();
    assert_eq!(bulk_result.total_permissions, 4);
    assert_eq!(bulk_result.granted_count, 2);
    assert_eq!(bulk_result.denied_count, 2);
    assert!(bulk_result.validation_time_ms > 0);
    
    // Check individual results
    assert_eq!(bulk_result.results.len(), 4);
    assert!(bulk_result.results.get("epsx:analytics:read").unwrap().granted);
    assert!(bulk_result.results.get("epsx:data:access").unwrap().granted);
    assert!(!bulk_result.results.get("admin:users:manage").unwrap().granted);
    assert!(!bulk_result.results.get("epsx:premium:features").unwrap().granted);
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

// ============================================================================
// CACHING TESTS
// ============================================================================

#[tokio::test]
async fn test_permission_caching_hit() {
    let authority = create_test_authority_with_cache(Duration::from_secs(300)).await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "epsx:analytics:read";
    
    setup_test_permissions(&authority.db_pool(), wallet, &[permission]).await;
    
    let context = create_test_context();
    
    // First call - should be a cache miss
    let result1 = authority.validate_permission(wallet, permission, &context).await;
    assert!(result1.is_ok());
    
    // Second call - should be a cache hit (faster)
    let start_time = std::time::Instant::now();
    let result2 = authority.validate_permission(wallet, permission, &context).await;
    let elapsed = start_time.elapsed();
    
    assert!(result2.is_ok());
    let validation_result = result2.unwrap();
    assert!(validation_result.granted);
    assert!(validation_result.cached);
    
    // Cache hit should be very fast
    assert!(elapsed.as_millis() < 50, "Cache hit took too long: {}ms", elapsed.as_millis());
    
    // Check cache stats
    let stats = authority.get_cache_stats().await;
    assert!(stats.permission_hits > 0);
    assert!(stats.permission_misses > 0);
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_permission_cache_expiry() {
    let authority = create_test_authority_with_cache(Duration::from_millis(100)).await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "epsx:analytics:read";
    
    setup_test_permissions(&authority.db_pool(), wallet, &[permission]).await;
    
    let context = create_test_context();
    
    // First call - cache miss
    let result1 = authority.validate_permission(wallet, permission, &context).await;
    assert!(result1.is_ok());
    
    // Wait for cache to expire
    sleep(Duration::from_millis(150)).await;
    
    // Second call - should be cache miss again due to expiry
    let result2 = authority.validate_permission(wallet, permission, &context).await;
    assert!(result2.is_ok());
    let validation_result = result2.unwrap();
    assert!(validation_result.granted);
    
    // Check that cache was refreshed
    let stats = authority.get_cache_stats().await;
    assert!(stats.permission_misses >= 2);
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_cache_invalidation() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let permission = "epsx:analytics:read";
    
    setup_test_permissions(&authority.db_pool(), wallet, &[permission]).await;
    
    let context = create_test_context();
    
    // First call to populate cache
    let result1 = authority.validate_permission(wallet, permission, &context).await;
    assert!(result1.is_ok());
    
    // Invalidate cache for this wallet
    authority.invalidate_wallet_cache(wallet).await;
    
    // Next call should be a cache miss
    let stats_before = authority.get_cache_stats().await;
    let result2 = authority.validate_permission(wallet, permission, &context).await;
    let stats_after = authority.get_cache_stats().await;
    
    assert!(result2.is_ok());
    assert!(stats_after.cache_invalidations > stats_before.cache_invalidations);
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_cache_size_limits() {
    let authority = create_test_authority_with_cache(Duration::from_secs(300)).await;
    
    // Create many test wallets to exceed cache size
    let mut test_wallets = Vec::new();
    for i in 0..50 {
        let wallet = format!("0x{:040x}", i);
        test_wallets.push(wallet.clone());
        setup_test_permissions(&authority.db_pool(), &wallet, &["epsx:analytics:read"]).await;
    }
    
    let context = create_test_context();
    
    // Access all wallets to populate cache
    for wallet in &test_wallets {
        let _ = authority.validate_permission(wallet, "epsx:analytics:read", &context).await;
    }
    
    // Check that cache management worked (no crashes, reasonable memory usage)
    let stats = authority.get_cache_stats().await;
    assert!(stats.permission_misses > 0);
    
    // Cleanup
    for wallet in &test_wallets {
        cleanup_test_data(&authority.db_pool(), wallet).await;
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_validation_performance() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    // Setup comprehensive permissions
    let permissions = [
        "epsx:analytics:read", "epsx:data:access", "admin:users:read",
        "admin:permission-groups:read", "epsx:premium:features"
    ];
    setup_test_permissions(&authority.db_pool(), wallet, &permissions).await;
    
    let context = create_test_context();
    let test_permissions: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
    
    // Measure bulk validation performance
    let start_time = std::time::Instant::now();
    let result = authority.bulk_validate_permissions(wallet, &test_permissions, &context).await;
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok());
    let bulk_result = result.unwrap();
    
    // Performance assertions
    assert!(elapsed.as_millis() < 500, "Bulk validation took too long: {}ms", elapsed.as_millis());
    assert!(bulk_result.validation_time_ms < 500);
    assert_eq!(bulk_result.granted_count, 5); // All permissions should be granted
    
    println!("Bulk validation of {} permissions took {}ms", 
        test_permissions.len(), elapsed.as_millis());
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test] 
async fn test_concurrent_validation_performance() {
    let authority = Arc::new(create_test_authority().await);
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    setup_test_permissions(&authority.db_pool(), wallet, &["epsx:analytics:read"]).await;
    
    let context = create_test_context();
    let num_concurrent = 10;
    let mut tasks = Vec::new();
    
    let start_time = std::time::Instant::now();
    
    // Spawn concurrent validation tasks
    for _i in 0..num_concurrent {
        let authority_clone = authority.clone();
        let wallet_clone = wallet.to_string();
        let context_clone = context.clone();
        
        let task = tokio::spawn(async move {
            authority_clone.validate_permission(
                &wallet_clone, 
                "epsx:analytics:read", 
                &context_clone
            ).await
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let mut success_count = 0;
    for task in tasks {
        if let Ok(Ok(result)) = task.await {
            if result.granted {
                success_count += 1;
            }
        }
    }
    
    let elapsed = start_time.elapsed();
    
    // All validations should succeed
    assert_eq!(success_count, num_concurrent);
    
    // Concurrent execution should be reasonably fast
    assert!(elapsed.as_millis() < 2000, "Concurrent validation took too long: {}ms", elapsed.as_millis());
    
    println!("Concurrent validation of {} requests took {}ms", 
        num_concurrent, elapsed.as_millis());
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_invalid_wallet_address() {
    let authority = create_test_authority().await;
    let invalid_wallet = "invalid-wallet-address";
    let context = create_test_context();
    
    let result = authority.validate_permission(invalid_wallet, "epsx:analytics:read", &context).await;
    
    // Should handle gracefully (either return error or deny permission)
    match result {
        Ok(validation_result) => {
            assert!(!validation_result.granted, "Invalid wallet should not be granted permissions");
        }
        Err(_) => {
            // Error is also acceptable for invalid wallet format
        }
    }
}

#[tokio::test]
async fn test_empty_permission_string() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    let context = create_test_context();
    
    let result = authority.validate_permission(wallet, "", &context).await;
    
    // Should handle empty permission gracefully
    match result {
        Ok(validation_result) => {
            assert!(!validation_result.granted, "Empty permission should not be granted");
        }
        Err(_) => {
            // Error is also acceptable for empty permission
        }
    }
}

// ============================================================================
// INTEGRATION WITH WEB3 PERMISSION SERVICE TESTS
// ============================================================================

#[tokio::test]
async fn test_get_wallet_permissions() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    let granted_permissions = ["epsx:analytics:read", "epsx:data:access", "admin:users:read"];
    setup_test_permissions(&authority.db_pool(), wallet, &granted_permissions).await;
    
    let result = authority.get_wallet_permissions(wallet).await;
    
    assert!(result.is_ok());
    let permissions = result.unwrap();
    assert!(permissions.len() >= 3);
    
    // Check that granted permissions are present
    let permission_names: Vec<String> = permissions.iter().map(|p| p.name.clone()).collect();
    for granted in &granted_permissions {
        assert!(permission_names.contains(&granted.to_string()), 
            "Permission {} not found in wallet permissions", granted);
    }
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

#[tokio::test]
async fn test_has_permission_convenience_method() {
    let authority = create_test_authority().await;
    let wallet = "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695";
    
    setup_test_permissions(&authority.db_pool(), wallet, &["epsx:analytics:read"]).await;
    
    // Test granted permission
    let result = authority.has_permission(wallet, "epsx:analytics:read").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Test denied permission
    let result = authority.has_permission(wallet, "admin:users:manage").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // Cleanup
    cleanup_test_data(&authority.db_pool(), wallet).await;
}

// ============================================================================
// CACHE WARMING TESTS
// ============================================================================

#[tokio::test]
async fn test_cache_warming() {
    let authority = create_test_authority().await;
    
    let test_wallets = vec![
        "0x742d35cc6abaac8b14a3780b5b0e11b2ce65d695".to_string(),
        "0x8ba1f109551bd432803012645hac136c6c0100c6".to_string(),
        "0x1234567890123456789012345678901234567890".to_string(),
    ];
    
    // Setup permissions for test wallets
    for wallet in &test_wallets {
        setup_test_permissions(&authority.db_pool(), wallet, &["epsx:analytics:read"]).await;
    }
    
    // Warm cache
    let result = authority.warm_cache(&test_wallets).await;
    assert!(result.is_ok());
    
    // Verify cache is warmed (subsequent calls should be faster)
    let context = create_test_context();
    for wallet in &test_wallets {
        let start_time = std::time::Instant::now();
        let validation_result = authority.validate_permission(wallet, "epsx:analytics:read", &context).await;
        let elapsed = start_time.elapsed();
        
        assert!(validation_result.is_ok());
        assert!(validation_result.unwrap().granted);
        assert!(elapsed.as_millis() < 50, "Warmed cache access should be fast");
    }
    
    // Check cache stats show hits
    let stats = authority.get_cache_stats().await;
    assert!(stats.permission_hits > 0);
    
    // Cleanup
    for wallet in &test_wallets {
        cleanup_test_data(&authority.db_pool(), wallet).await;
    }
}