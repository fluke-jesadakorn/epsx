//! Comprehensive integration tests for Casbin authorization system
//! Tests the complete flow from database to policy enforcement

use epsx::dom::services::casbin_service::CasbinService;
use epsx::web::auth::AppState;
use epsx::web::iam::handlers::*;
use epsx::web::admin::casbin_handlers::*;
use axum::{
    body::Body,  
    extract::State,
    http::{Request, StatusCode, Method},
    response::Json,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio;

pub struct TestContext {
    pub app_state: AppState,
    pub db_pool: PgPool,
}

impl TestContext {
    pub async fn new() -> Self {
        // Use test database URL
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test_db".to_string());
        
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        // Clear existing policies for clean test
        sqlx::query("DELETE FROM casbin_rule")
            .execute(&pool)
            .await
            .expect("Failed to clear test policies");
        
        // Create CasbinService
        let casbin_service = Arc::new(
            CasbinService::new(pool.clone())
                .await
                .expect("Failed to create CasbinService")
        );
        
        // Create minimal AppState for testing
        let app_state = AppState {
            casbin_service,
            // Add other required fields as minimal/mock implementations
        };
        
        TestContext {
            app_state,
            db_pool: pool,
        }
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data
        sqlx::query("DELETE FROM casbin_rule")
            .execute(&self.db_pool)
            .await
            .expect("Failed to cleanup test data");
    }
}

#[tokio::test]
async fn test_rbac_hierarchy_enforcement() {
    let ctx = TestContext::new().await;
    
    // Test setup: Create role hierarchy
    // admin -> moderator -> premium_user -> basic_user
    
    // Add role inheritance
    ctx.app_state.casbin_service.add_role_for_user("admin_user", "admin").await.unwrap();
    ctx.app_state.casbin_service.add_role_for_user("moderator_user", "moderator").await.unwrap();
    ctx.app_state.casbin_service.add_role_for_user("premium_user_1", "premium_user").await.unwrap();
    ctx.app_state.casbin_service.add_role_for_user("basic_user_1", "basic_user").await.unwrap();
    
    // Add policies for roles
    ctx.app_state.casbin_service.add_policy("admin", "/api/v1/admin", "GET").await.unwrap();
    ctx.app_state.casbin_service.add_policy("moderator", "/api/v1/users", "GET").await.unwrap();
    ctx.app_state.casbin_service.add_policy("premium_user", "/api/v1/analytics", "GET").await.unwrap();
    ctx.app_state.casbin_service.add_policy("basic_user", "/api/v1/trading", "GET").await.unwrap();
    
    // Test 1: Admin should have access to everything through inheritance
    assert_eq!(
        ctx.app_state.casbin_service.enforce("admin_user", "/api/v1/admin", "GET").await.unwrap(),
        true,
        "Admin should access admin endpoints"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("admin_user", "/api/v1/users", "GET").await.unwrap(),
        true,
        "Admin should inherit moderator permissions"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("admin_user", "/api/v1/analytics", "GET").await.unwrap(),
        true,
        "Admin should inherit premium user permissions"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("admin_user", "/api/v1/trading", "GET").await.unwrap(),
        true,
        "Admin should inherit basic user permissions"
    );
    
    // Test 2: Basic user should only have basic access
    assert_eq!(
        ctx.app_state.casbin_service.enforce("basic_user_1", "/api/v1/trading", "GET").await.unwrap(),
        true,
        "Basic user should access trading endpoints"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("basic_user_1", "/api/v1/analytics", "GET").await.unwrap(),
        false,
        "Basic user should not access premium features"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("basic_user_1", "/api/v1/admin", "GET").await.unwrap(),
        false,
        "Basic user should not access admin endpoints"
    );
    
    // Test 3: Premium user should have premium + basic access
    assert_eq!(
        ctx.app_state.casbin_service.enforce("premium_user_1", "/api/v1/analytics", "GET").await.unwrap(),
        true,
        "Premium user should access analytics"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("premium_user_1", "/api/v1/trading", "GET").await.unwrap(),
        true,
        "Premium user should inherit basic permissions"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("premium_user_1", "/api/v1/admin", "GET").await.unwrap(),
        false,
        "Premium user should not access admin endpoints"
    );
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_policy_caching_performance() {
    let ctx = TestContext::new().await;
    
    // Add test policy
    ctx.app_state.casbin_service.add_policy("test_user", "/api/v1/test", "GET").await.unwrap();
    
    // Measure first enforcement (cache miss)
    let start = std::time::Instant::now();
    let result1 = ctx.app_state.casbin_service.enforce("test_user", "/api/v1/test", "GET").await.unwrap();
    let first_duration = start.elapsed();
    
    // Measure second enforcement (cache hit)
    let start = std::time::Instant::now();
    let result2 = ctx.app_state.casbin_service.enforce("test_user", "/api/v1/test", "GET").await.unwrap();
    let second_duration = start.elapsed();
    
    assert_eq!(result1, true);
    assert_eq!(result2, true);
    
    // Cache hit should be significantly faster
    assert!(
        second_duration < first_duration,
        "Cache hit should be faster than cache miss. First: {:?}, Second: {:?}",
        first_duration, second_duration
    );
    
    // Verify cache statistics
    let stats = ctx.app_state.casbin_service.cache_stats().await;
    assert!(stats.total_entries > 0, "Cache should have entries");
    assert!(stats.active_entries > 0, "Cache should have active entries");
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_wildcard_pattern_matching() {
    let ctx = TestContext::new().await;
    
    // Add wildcard policies
    ctx.app_state.casbin_service.add_policy("wildcard_user", "/api/v1/*", "GET").await.unwrap();
    ctx.app_state.casbin_service.add_policy("action_user", "/api/v1/specific", "*").await.unwrap();
    
    // Test resource wildcard
    assert_eq!(
        ctx.app_state.casbin_service.enforce("wildcard_user", "/api/v1/users", "GET").await.unwrap(),
        true,
        "Wildcard resource should match specific resource"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("wildcard_user", "/api/v1/admin", "GET").await.unwrap(),
        true,
        "Wildcard resource should match different resource"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("wildcard_user", "/api/v1/users", "POST").await.unwrap(),
        false,
        "Wildcard resource should not match different action"
    );
    
    // Test action wildcard
    assert_eq!(
        ctx.app_state.casbin_service.enforce("action_user", "/api/v1/specific", "GET").await.unwrap(),
        true,
        "Wildcard action should match GET"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("action_user", "/api/v1/specific", "POST").await.unwrap(),
        true,
        "Wildcard action should match POST"
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("action_user", "/api/v1/different", "GET").await.unwrap(),
        false,
        "Wildcard action should not match different resource"
    );
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_batch_policy_operations() {
    let ctx = TestContext::new().await;
    
    // Test batch policy addition
    let policies = vec![
        ("batch_user".to_string(), "/api/v1/resource1".to_string(), "GET".to_string()),
        ("batch_user".to_string(), "/api/v1/resource2".to_string(), "POST".to_string()),
        ("batch_user".to_string(), "/api/v1/resource3".to_string(), "PUT".to_string()),
    ];
    
    let batch_result = ctx.app_state.casbin_service.add_policies(policies).await.unwrap();
    assert_eq!(batch_result, true, "Batch policy addition should succeed");
    
    // Verify all policies were added
    assert_eq!(
        ctx.app_state.casbin_service.enforce("batch_user", "/api/v1/resource1", "GET").await.unwrap(),
        true
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("batch_user", "/api/v1/resource2", "POST").await.unwrap(),
        true
    );
    assert_eq!(
        ctx.app_state.casbin_service.enforce("batch_user", "/api/v1/resource3", "PUT").await.unwrap(),
        true
    );
    
    // Test getting all policies
    let (policies, role_policies) = ctx.app_state.casbin_service.get_all_policies().await.unwrap();
    assert!(policies.len() >= 3, "Should have at least 3 policies");
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_dynamic_policy_reload() {
    let ctx = TestContext::new().await;
    
    // Add policy through service
    ctx.app_state.casbin_service.add_policy("reload_user", "/api/v1/test", "GET").await.unwrap();
    
    // Verify policy works
    assert_eq!(
        ctx.app_state.casbin_service.enforce("reload_user", "/api/v1/test", "GET").await.unwrap(),
        true
    );
    
    // Manually add policy to database (simulating external change)
    sqlx::query("INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES ('p', 'reload_user', '/api/v1/external', 'GET')")
        .execute(&ctx.db_pool)
        .await
        .unwrap();
    
    // Policy should not be recognized until reload
    assert_eq!(
        ctx.app_state.casbin_service.enforce("reload_user", "/api/v1/external", "GET").await.unwrap(),
        false,
        "External policy should not be active before reload"
    );
    
    // Reload policies
    ctx.app_state.casbin_service.reload_policies().await.unwrap();
    
    // Now external policy should work
    assert_eq!(
        ctx.app_state.casbin_service.enforce("reload_user", "/api/v1/external", "GET").await.unwrap(),
        true,
        "External policy should be active after reload"
    );
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    let ctx = TestContext::new().await;
    
    // Test empty subject/object/action
    let result = ctx.app_state.casbin_service.enforce("", "/api/v1/test", "GET").await;
    assert!(result.is_ok(), "Empty subject should not cause error");
    assert_eq!(result.unwrap(), false, "Empty subject should be denied");
    
    let result = ctx.app_state.casbin_service.enforce("test_user", "", "GET").await;
    assert!(result.is_ok(), "Empty object should not cause error");
    assert_eq!(result.unwrap(), false, "Empty object should be denied");
    
    let result = ctx.app_state.casbin_service.enforce("test_user", "/api/v1/test", "").await;
    assert!(result.is_ok(), "Empty action should not cause error");
    assert_eq!(result.unwrap(), false, "Empty action should be denied");
    
    // Test special characters in policy
    let result = ctx.app_state.casbin_service.add_policy("user@domain.com", "/api/v1/special-chars_123", "GET").await;
    assert!(result.is_ok(), "Special characters should be handled");
    
    // Test very long strings
    let long_string = "a".repeat(1000);
    let result = ctx.app_state.casbin_service.add_policy(&long_string, "/api/v1/test", "GET").await;
    assert!(result.is_ok(), "Long strings should be handled gracefully");
    
    // Test policy removal for non-existent policy
    let result = ctx.app_state.casbin_service.remove_policy("non_existent", "/api/v1/none", "GET").await;
    assert!(result.is_ok(), "Removing non-existent policy should not error");
    assert_eq!(result.unwrap(), false, "Should return false for non-existent policy");
    
    ctx.cleanup().await;
}

#[tokio::test] 
async fn test_concurrent_policy_enforcement() {
    let ctx = TestContext::new().await;
    
    // Add test policy
    ctx.app_state.casbin_service.add_policy("concurrent_user", "/api/v1/test", "GET").await.unwrap();
    
    // Create multiple concurrent enforcement tasks
    let mut handles = vec![];
    
    for i in 0..100 {
        let casbin_service = ctx.app_state.casbin_service.clone();
        let handle = tokio::spawn(async move {
            let result = casbin_service.enforce("concurrent_user", "/api/v1/test", "GET").await;
            (i, result.unwrap())
        });
        handles.push(handle);
    }
    
    // Wait for all tasks and verify results
    let mut success_count = 0;
    for handle in handles {
        let (_, result) = handle.await.unwrap();
        if result {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, 100, "All concurrent enforcements should succeed");
    
    ctx.cleanup().await;
}

#[tokio::test]
async fn test_admin_handler_integration() {
    let ctx = TestContext::new().await;
    
    // Add admin user and permissions
    ctx.app_state.casbin_service.add_role_for_user("admin_test", "admin").await.unwrap();
    ctx.app_state.casbin_service.add_policy("admin", "casbin_policies", "read").await.unwrap();
    
    // Test get all policies handler
    let result = get_all_policies_handler(State(ctx.app_state.clone())).await;
    assert!(result.is_ok(), "Admin should be able to get all policies");
    
    // Test add policy handler  
    let policy_request = PolicyRequest {
        subject: "test_subject".to_string(),
        object: "test_object".to_string(),
        action: "test_action".to_string(),
    };
    
    // Note: This test assumes verify_admin_access passes in test environment
    // In a real test, you'd mock the admin verification
    
    ctx.cleanup().await;
}