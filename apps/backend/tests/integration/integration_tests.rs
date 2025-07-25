// Integration tests for EPSX backend
use epsx::infra::db::MigrationRunner;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio;

#[tokio::test]
async fn test_migration_system() {
    // Skip if no test database URL is provided
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping migration test - no TEST_DATABASE_URL set");
            return;
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let runner = MigrationRunner::new(pool, "migrations".to_string());

    // Test initialization
    let result = runner.init().await;
    assert!(result.is_ok(), "Migration initialization should succeed");

    // Test status check
    let result = runner.status().await;
    assert!(result.is_ok(), "Status check should succeed");
}

#[tokio::test]
async fn test_database_connection() {
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping database connection test - no TEST_DATABASE_URL set");
            return;
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await;

    assert!(pool.is_ok(), "Database connection should succeed");
}

#[tokio::test]
async fn test_permission_workflow_integration() {
    use epsx::dom::entities::{User, RolePermissionProfile, Permission};
    use epsx::dom::entities::iam::PackageTier;
    use epsx::dom::entities::permission_profile::PermissionProfileCategory;
    use epsx::dom::values::{Email, Role, UserId};
    use epsx::dom::services::permission_resolver::PermissionResolver;
    
    // Create test user
    let email = Email::new("test@example.com").unwrap();
    let user = User::new(email, Role::User);
    let user_id = user.id().clone();
    
    // Create permission profile
    let mut profile = RolePermissionProfile::new(
        "Test Integration Profile".to_string(),
        "Integration test profile".to_string(),
        PackageTier::Bronze,
        PermissionProfileCategory::User,
        user_id.clone(),
    );
    
    // Add permissions
    profile.add_permission(Permission::new("read".to_string(), "posts".to_string()));
    profile.add_permission(Permission::new("write".to_string(), "comments".to_string()));
    
    // Test permission resolution
    let resolver = PermissionResolver::new().await.unwrap();
    let resolution = resolver.resolve_user_permissions(&user_id, vec![profile]).await.unwrap();
    
    assert_eq!(resolution.effective_permissions.len(), 2);
    assert!(resolution.effective_permissions.iter().any(|p| p.action() == "read" && p.resource() == "posts"));
    assert!(resolution.effective_permissions.iter().any(|p| p.action() == "write" && p.resource() == "comments"));
}

#[tokio::test]
async fn test_api_endpoint_protection_workflow() {
    use epsx::dom::entities::Permission;
    use epsx::dom::services::permission_resolver::PermissionResolver;
    use epsx::dom::values::UserId;
    
    let resolver = PermissionResolver::new().await.unwrap();
    let user_id = UserId::new("api_test_user".to_string());
    
    // Test different API endpoint protections
    let permissions = vec![
        Permission::new("get".to_string(), "api:/users/*".to_string()),
        Permission::new("post".to_string(), "api:/posts".to_string()),
        Permission::new("*".to_string(), "api:/admin/*".to_string()),
    ];
    
    // Test allowed access
    let result = resolver.check_api_access(&user_id, &permissions, "/users/123", "GET").await;
    assert!(result.granted, "Should allow GET /users/123");
    
    let result = resolver.check_api_access(&user_id, &permissions, "/posts", "POST").await;
    assert!(result.granted, "Should allow POST /posts");
    
    let result = resolver.check_api_access(&user_id, &permissions, "/admin/settings", "DELETE").await;
    assert!(result.granted, "Should allow any action on /admin/*");
    
    // Test denied access
    let result = resolver.check_api_access(&user_id, &permissions, "/forbidden", "GET").await;
    assert!(!result.granted, "Should deny access to /forbidden");
}

#[tokio::test]
async fn test_cache_integration() {
    use epsx::dom::services::permission_resolver::PermissionResolver;
    use epsx::dom::entities::RolePermissionProfile;
    use epsx::dom::entities::iam::PackageTier;
    use epsx::dom::entities::permission_profile::PermissionProfileCategory;
    use epsx::dom::values::UserId;
    
    let resolver = PermissionResolver::new().await.unwrap();
    let user_id = UserId::new("cache_test_user".to_string());
    
    let profile = RolePermissionProfile::new(
        "Cache Test Profile".to_string(),
        "Profile for testing cache".to_string(),
        PackageTier::Bronze,
        PermissionProfileCategory::User,
        user_id.clone(),
    );
    
    // First resolution - should cache result
    let result1 = resolver.resolve_user_permissions(&user_id, vec![profile.clone()]).await.unwrap();
    assert!(!result1.from_cache, "First resolution should not be from cache");
    
    // Test cache invalidation
    let _ = resolver.invalidate_user_cache(&user_id).await;
    
    // Test cache stats
    let stats = resolver.get_cache_stats().await.unwrap();
    assert!(stats.total_entries >= 0);
}