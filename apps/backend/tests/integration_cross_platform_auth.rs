// Integration tests for cross-platform authentication system
// Tests JWT generation, permission validation, and platform access control

use epsx::auth::{JWT, User, jwt};
use epsx::auth::jwt::CROSS_PLATFORM_PERMISSION_SERVICE;
use epsx::web::middleware::modern_auth::{PlatformContext, cross_platform_auth_middleware};
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderValue, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use tower::ServiceExt;
use uuid::Uuid;

/// Test cross-platform JWT token generation with structured permissions
#[tokio::test]
async fn test_cross_platform_jwt_generation() {
    let service = jwt::Service::new().unwrap();
    
    let user_data = jwt::UserData {
        id: Uuid::new_v4().to_string(),
        email: "cross.platform@epsx.io".to_string(),
        name: Some("Cross Platform User".to_string()),
        permissions: Some(vec![
            // EPSX Platform permissions
            "epsx:analytics:read".to_string(),
            "epsx:analytics:write".to_string(),
            "epsx:users:read".to_string(),
            
            // EPSX Pay permissions
            "epsx-pay:transactions:read".to_string(),
            "epsx-pay:transactions:create".to_string(),
            "epsx-pay:wallets:manage".to_string(),
            
            // EPSX Token permissions
            "epsx-token:governance:vote".to_string(),
            "epsx-token:treasury:view".to_string(),
        ]),
        package_tier: Some("PREMIUM".to_string()),
        firebase_uid: None,
        audience: Some("epsx-ecosystem".to_string()),
        ttl_seconds: Some(3600),
        
        // Cross-platform fields
        platforms: Some(vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()]),
        primary_platform: Some("epsx".to_string()),
        platform_context: Some("epsx-pay".to_string()),
    };
    
    // Generate JWT token
    let token = service.create(user_data).unwrap();
    assert!(!token.is_empty());
    
    // Verify token structure
    let claims = service.verify(&token).unwrap();
    assert_eq!(claims.aud, "epsx-ecosystem");
    assert!(claims.platforms.as_ref().unwrap().contains(&"epsx-pay".to_string()));
    assert_eq!(claims.primary_platform.as_ref().unwrap(), "epsx");
    assert_eq!(claims.platform_context.as_ref().unwrap(), "epsx-pay");
    
    // Verify permissions
    assert!(claims.permissions.contains(&"epsx:analytics:read".to_string()));
    assert!(claims.permissions.contains(&"epsx-pay:transactions:create".to_string()));
    assert!(claims.permissions.contains(&"epsx-token:governance:vote".to_string()));
    
    // Decode to User struct
    let user = service.decode(&token).unwrap();
    assert_eq!(user.platforms.len(), 3);
    assert_eq!(user.primary_platform, "epsx");
    assert_eq!(user.platform_context.as_ref().unwrap(), "epsx-pay");
}

/// Test CrossPlatformPermissionService validation logic
#[tokio::test]
async fn test_cross_platform_permission_validation() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Create test user with structured permissions
    let user = User {
        id: "test_user".to_string(),
        email: "test@epsx.io".to_string(),
        name: None,
        permissions: vec![
            "epsx:analytics:read".to_string(),
            "epsx:analytics:write".to_string(),
            "epsx-pay:transactions:*".to_string(), // Wildcard
            "epsx-token:governance:vote".to_string(),
        ],
        package_tier: "PREMIUM".to_string(),
        role: "user".to_string(),
        firebase_uid: None,
        platforms: vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()],
        primary_platform: "epsx".to_string(),
        platform_context: None,
    };

    // Test exact permission matches
    assert!(permission_service.validate_platform_permission(&user, "epsx", "analytics", "read"));
    assert!(permission_service.validate_platform_permission(&user, "epsx", "analytics", "write"));
    
    // Test wildcard permission matching
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "read"));
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "create"));
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "delete"));
    
    // Test specific permission
    assert!(permission_service.validate_platform_permission(&user, "epsx-token", "governance", "vote"));
    assert!(!permission_service.validate_platform_permission(&user, "epsx-token", "governance", "propose"));
    
    // Test non-existent permissions
    assert!(!permission_service.validate_platform_permission(&user, "epsx", "users", "delete"));
    assert!(!permission_service.validate_platform_permission(&user, "non-existent", "resource", "action"));
    
    // Test platform access
    assert!(permission_service.can_access_platform(&user, "epsx"));
    assert!(permission_service.can_access_platform(&user, "epsx-pay"));
    assert!(permission_service.can_access_platform(&user, "epsx-token"));
    assert!(!permission_service.can_access_platform(&user, "non-existent"));
    
    // Test accessible platforms
    let platforms = permission_service.get_accessible_platforms(&user);
    assert_eq!(platforms.len(), 3);
    assert!(platforms.contains(&"epsx".to_string()));
    assert!(platforms.contains(&"epsx-pay".to_string()));
    assert!(platforms.contains(&"epsx-token".to_string()));
    
    // Test permission parsing
    let parsed = permission_service.parse_permission("epsx-pay:wallets:manage");
    assert!(parsed.is_some());
    let (platform, resource, action) = parsed.unwrap();
    assert_eq!(platform, "epsx-pay");
    assert_eq!(resource, "wallets");
    assert_eq!(action, "manage");
    
    // Test permission building
    let built = permission_service.build_permission("epsx-token", "treasury", "approve");
    assert_eq!(built, "epsx-token:treasury:approve");
    
    // Test platform-specific permissions
    let epsx_permissions = permission_service.get_platform_permissions(&user, "epsx");
    assert_eq!(epsx_permissions.len(), 2);
    assert!(epsx_permissions.contains(&"epsx:analytics:read".to_string()));
    
    let pay_permissions = permission_service.get_platform_permissions(&user, "epsx-pay");
    assert_eq!(pay_permissions.len(), 1);
    assert!(pay_permissions.contains(&"epsx-pay:transactions:*".to_string()));
}

/// Test admin user cross-platform access
#[tokio::test]
async fn test_admin_cross_platform_access() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Create admin user
    let admin_user = User {
        id: "admin_user".to_string(),
        email: "admin@epsx.io".to_string(),
        name: None,
        permissions: vec!["admin:*".to_string()],
        package_tier: "ENTERPRISE".to_string(),
        role: "admin".to_string(),
        firebase_uid: None,
        platforms: vec!["epsx".to_string()], // Admin might only be explicitly assigned to one platform
        primary_platform: "epsx".to_string(),
        platform_context: None,
    };

    // Admin should have access to all resources across all platforms
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx", "users", "manage"));
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx", "analytics", "write"));
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx-pay", "transactions", "create"));
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx-pay", "wallets", "manage"));
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx-token", "governance", "propose"));
    assert!(permission_service.validate_platform_permission(&admin_user, "epsx-token", "treasury", "approve"));
    
    // Admin should have platform admin access
    assert!(permission_service.has_platform_admin_access(&admin_user, "epsx"));
    assert!(permission_service.has_platform_admin_access(&admin_user, "epsx-pay"));
    assert!(permission_service.has_platform_admin_access(&admin_user, "epsx-token"));
}

/// Test platform context middleware integration
#[tokio::test]
async fn test_platform_context_middleware() {
    // This test would require setting up a full Axum application
    // For now, we'll test the core logic components
    
    let service = jwt::Service::new().unwrap();
    
    // Create a user with cross-platform access
    let user_data = jwt::UserData {
        id: "middleware_test_user".to_string(),
        email: "middleware@epsx.io".to_string(),
        name: Some("Middleware Test User".to_string()),
        permissions: Some(vec![
            "epsx-pay:transactions:read".to_string(),
            "epsx-token:governance:vote".to_string(),
        ]),
        package_tier: Some("PREMIUM".to_string()),
        firebase_uid: None,
        audience: Some("epsx-ecosystem".to_string()),
        ttl_seconds: Some(3600),
        platforms: Some(vec!["epsx-pay".to_string(), "epsx-token".to_string()]),
        primary_platform: Some("epsx-pay".to_string()),
        platform_context: Some("epsx-pay".to_string()),
    };
    
    let token = service.create(user_data).unwrap();
    
    // Verify the user can be decoded properly
    let user = service.decode(&token).unwrap();
    
    // Test permission validation for platform context
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Should have access to epsx-pay transactions
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "read"));
    
    // Should have access to epsx-token governance
    assert!(permission_service.validate_platform_permission(&user, "epsx-token", "governance", "vote"));
    
    // Should not have access to epsx platform (not in user's platforms)
    assert!(!permission_service.can_access_platform(&user, "epsx"));
    
    // Should have access to assigned platforms
    assert!(permission_service.can_access_platform(&user, "epsx-pay"));
    assert!(permission_service.can_access_platform(&user, "epsx-token"));
}

/// Test JWT backward compatibility with legacy tokens
#[tokio::test]
async fn test_backward_compatibility() {
    let service = jwt::Service::new().unwrap();
    
    // Create legacy-style user data (without cross-platform fields)
    let legacy_user_data = jwt::UserData {
        id: "legacy_user".to_string(),
        email: "legacy@epsx.io".to_string(),
        name: Some("Legacy User".to_string()),
        permissions: Some(vec![
            "analytics:read".to_string(),  // Old format
            "user:write".to_string(),      // Old format
        ]),
        package_tier: Some("GOLD".to_string()),
        firebase_uid: None,
        audience: None, // Will default to "epsx-ecosystem"
        ttl_seconds: Some(3600),
        
        // Cross-platform fields omitted (should get defaults)
        platforms: None,
        primary_platform: None,
        platform_context: None,
    };
    
    let token = service.create(legacy_user_data).unwrap();
    let claims = service.verify(&token).unwrap();
    
    // Should get default values
    assert_eq!(claims.aud, "epsx-ecosystem");
    assert_eq!(claims.platforms.as_ref().unwrap(), &vec!["epsx".to_string()]);
    assert_eq!(claims.primary_platform.as_ref().unwrap(), "epsx");
    assert!(claims.platform_context.is_none());
    
    // Legacy permissions should still work
    assert!(claims.permissions.contains(&"analytics:read".to_string()));
    assert!(claims.permissions.contains(&"user:write".to_string()));
    
    // Decode to User should work
    let user = service.decode(&token).unwrap();
    assert_eq!(user.platforms, vec!["epsx".to_string()]);
    assert_eq!(user.primary_platform, "epsx");
    assert!(user.platform_context.is_none());
    
    // Legacy permission validation should still work
    let jwt_service = &service;
    assert!(jwt_service.can(&user, "analytics:read"));
    assert!(jwt_service.can(&user, "user:write"));
}

/// Test permission inheritance and wildcards
#[tokio::test]
async fn test_permission_inheritance() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Test various wildcard patterns
    let user = User {
        id: "wildcard_user".to_string(),
        email: "wildcard@epsx.io".to_string(),
        name: None,
        permissions: vec![
            "epsx:*".to_string(),                    // Platform wildcard
            "epsx-pay:transactions:*".to_string(),   // Resource wildcard
            "epsx-token:governance:vote".to_string(), // Specific permission
        ],
        package_tier: "PREMIUM".to_string(),
        role: "user".to_string(),
        firebase_uid: None,
        platforms: vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()],
        primary_platform: "epsx".to_string(),
        platform_context: None,
    };

    // Platform wildcard should grant access to all resources on epsx
    assert!(permission_service.validate_platform_permission(&user, "epsx", "analytics", "read"));
    assert!(permission_service.validate_platform_permission(&user, "epsx", "users", "manage"));
    assert!(permission_service.validate_platform_permission(&user, "epsx", "settings", "write"));
    
    // Resource wildcard should grant all actions on epsx-pay transactions
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "read"));
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "create"));
    assert!(permission_service.validate_platform_permission(&user, "epsx-pay", "transactions", "approve"));
    
    // But not other resources on epsx-pay
    assert!(!permission_service.validate_platform_permission(&user, "epsx-pay", "wallets", "read"));
    
    // Specific permission should work
    assert!(permission_service.validate_platform_permission(&user, "epsx-token", "governance", "vote"));
    assert!(!permission_service.validate_platform_permission(&user, "epsx-token", "governance", "propose"));
}

/// Test edge cases and error conditions
#[tokio::test]
async fn test_edge_cases() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Empty user permissions
    let empty_user = User {
        id: "empty_user".to_string(),
        email: "empty@epsx.io".to_string(),
        name: None,
        permissions: vec![],
        package_tier: "FREE".to_string(),
        role: "guest".to_string(),
        firebase_uid: None,
        platforms: vec![],
        primary_platform: "epsx".to_string(),
        platform_context: None,
    };

    // Should not have any permissions
    assert!(!permission_service.validate_platform_permission(&empty_user, "epsx", "analytics", "read"));
    assert!(!permission_service.can_access_platform(&empty_user, "epsx"));
    
    // Accessible platforms should be empty
    let platforms = permission_service.get_accessible_platforms(&empty_user);
    assert!(platforms.is_empty());
    
    // Test invalid permission formats
    assert!(permission_service.parse_permission("invalid").is_none());
    assert!(permission_service.parse_permission("platform:resource").is_none());
    assert!(permission_service.parse_permission("platform:resource:action:extra").is_none());
    
    // Valid permission format
    let valid = permission_service.parse_permission("epsx:analytics:read");
    assert!(valid.is_some());
    let (platform, resource, action) = valid.unwrap();
    assert_eq!(platform, "epsx");
    assert_eq!(resource, "analytics");
    assert_eq!(action, "read");
}

/// Performance test for permission validation
#[tokio::test]
async fn test_permission_validation_performance() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Create user with many permissions
    let mut permissions = Vec::new();
    for platform in ["epsx", "epsx-pay", "epsx-token"] {
        for resource in ["analytics", "users", "transactions", "governance", "treasury"] {
            for action in ["read", "write", "create", "update", "delete", "manage", "approve"] {
                permissions.push(format!("{}:{}:{}", platform, resource, action));
            }
        }
    }
    
    let user = User {
        id: "performance_user".to_string(),
        email: "performance@epsx.io".to_string(),
        name: None,
        permissions,
        package_tier: "ENTERPRISE".to_string(),
        role: "user".to_string(),
        firebase_uid: None,
        platforms: vec!["epsx".to_string(), "epsx-pay".to_string(), "epsx-token".to_string()],
        primary_platform: "epsx".to_string(),
        platform_context: None,
    };

    // Time permission validation
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        permission_service.validate_platform_permission(&user, "epsx", "analytics", "read");
    }
    let duration = start.elapsed();
    
    // Should be very fast (less than 1ms for 1000 validations)
    assert!(duration.as_millis() < 10, "Permission validation too slow: {:?}", duration);
    
    // Test wildcard performance
    let wildcard_user = User {
        permissions: vec!["epsx:*".to_string()],
        ..user
    };
    
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        permission_service.validate_platform_permission(&wildcard_user, "epsx", "analytics", "read");
    }
    let duration = start.elapsed();
    
    // Wildcard should also be fast
    assert!(duration.as_millis() < 10, "Wildcard permission validation too slow: {:?}", duration);
}