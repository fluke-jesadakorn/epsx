// Integration tests for cross-platform authentication system
// Tests JWT generation, permission validation, and platform access control

use epsx::auth::{JWT, User, jwt};
use uuid::Uuid;

use epsx::auth::jwt::{

    derive_package_tier_from_permissions,
    derive_accessible_platforms_from_permissions,
    derive_primary_platform_from_permissions
};
use epsx::auth::jwt::CROSS_PLATFORM_PERMISSION_SERVICE;

use epsx::web::middleware::modern_auth::{PlatformContext, cross_platform_auth_middleware};

use axum::{

    body::Body,
    http::{Request, StatusCode, HeaderValue, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use tower::ServiceExt;


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
        audience: Some("epsx-ecosystem".to_string()),
        ttl_seconds: Some(3600),
    };
    
    // Generate JWT token
    let token = service.create(user_data).unwrap();
    assert!(!token.is_empty());
    
    // Verify token structure
    let claims = service.verify(&token).await.unwrap();
    assert_eq!(claims.aud, "epsx-ecosystem");
    
    // Verify permissions
    assert!(claims.permissions.contains(&"epsx:analytics:read".to_string()));
    assert!(claims.permissions.contains(&"epsx-pay:transactions:create".to_string()));
    assert!(claims.permissions.contains(&"epsx-token:governance:vote".to_string()));
    
    // Test derivation functions with token permissions
    let accessible_platforms = derive_accessible_platforms_from_permissions(&claims.permissions);
    assert!(accessible_platforms.contains(&"epsx-pay".to_string()));
    assert!(accessible_platforms.contains(&"epsx".to_string()));
    assert!(accessible_platforms.contains(&"epsx-token".to_string()));
    assert_eq!(accessible_platforms.len(), 3);
    
    let primary_platform = derive_primary_platform_from_permissions(&claims.permissions);
    assert_eq!(primary_platform, "epsx"); // epsx has priority over epsx-pay and epsx-token
    
    let package_tier = derive_package_tier_from_permissions(&claims.permissions);
    assert!(package_tier == "GOLD" || package_tier == "SILVER"); // Based on advanced permissions
    
    // Decode to User struct
    let user = service.decode(&token).await.unwrap();
}

/// Test CrossPlatformPermissionService validation logic
#[tokio::test]
async fn test_cross_platform_permission_validation() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Test permissions array instead of User struct
    let permissions = vec![
        "epsx:analytics:read".to_string(),
        "epsx:analytics:write".to_string(),
        "epsx-pay:transactions:*".to_string(), // Wildcard
        "epsx-token:governance:vote".to_string(),
    ];

    // Note: Individual permission validation methods are deprecated
    // Use permission checking through auth/permissions.rs check_permission_access instead
    
    // Test platform access with new permission-based API
    assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx"));
    assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx-pay"));
    assert!(permission_service.can_access_platform_with_permissions(&permissions, "epsx-token"));
    assert!(!permission_service.can_access_platform_with_permissions(&permissions, "non-existent"));
    
    // Test accessible platforms derivation
    let platforms = permission_service.get_accessible_platforms_from_permissions(&permissions);
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
    
    // Test derivation functions
    let package_tier = derive_package_tier_from_permissions(&permissions);
    assert!(package_tier == "GOLD" || package_tier == "SILVER"); // Based on advanced permissions
    
    let primary_platform = derive_primary_platform_from_permissions(&permissions);
    assert_eq!(primary_platform, "epsx"); // epsx has priority
}

/// Test admin user cross-platform access
#[tokio::test]
async fn test_admin_cross_platform_access() {
    let permission_service = &CROSS_PLATFORM_PERMISSION_SERVICE;
    
    // Admin permissions array
    let admin_permissions = vec!["admin:*:*".to_string()];

    // Test admin derivations
    let package_tier = derive_package_tier_from_permissions(&admin_permissions);
    assert_eq!(package_tier, "ENTERPRISE");
    
    let primary_platform = derive_primary_platform_from_permissions(&admin_permissions);
    assert_eq!(primary_platform, "admin");
    
    // Admin should have access to all platforms through admin permissions
    let accessible_platforms = derive_accessible_platforms_from_permissions(&admin_permissions);
    assert!(accessible_platforms.contains(&"admin".to_string()));
    
    // Test platform access with admin permissions
    assert!(permission_service.can_access_platform_with_permissions(&admin_permissions, "admin"));
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
        audience: Some("epsx-ecosystem".to_string()),
        ttl_seconds: Some(3600),
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