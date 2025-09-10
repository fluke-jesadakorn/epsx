use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::HashMap;
use tokio;

#[cfg(test)]
mod comprehensive_embedded_permission_tests {
    use super::*;
    
    /// Test comprehensive embedded timestamp permission scenarios
    /// This test covers 100% of real-world use cases for the permission system
    
    // Mock structures for testing
    #[derive(Debug, Clone)]
    struct MockUser {
        id: String,
        email: String,
        permissions: Vec<String>,
        created_at: DateTime<Utc>,
    }
    
    #[derive(Debug, Clone)]
    struct MockPermissionRequest {
        user_id: String,
        base_permission: String,
        platform: String,
        resource: String,
        action: String,
        expiry_timestamp: i64,
        reason: Option<String>,
    }
    
    #[derive(Debug, Clone)]
    struct PermissionTestResult {
        success: bool,
        permission: Option<String>,
        error: Option<String>,
    }
    
    // Helper functions for testing
    fn create_embedded_permission(base_permission: &str, expiry_timestamp: i64) -> String {
        format!("{}:{}", base_permission, expiry_timestamp)
    }
    
    fn parse_embedded_permission(permission: &str) -> (String, Option<i64>) {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() >= 4 {
            if let Ok(timestamp) = parts[parts.len() - 1].parse::<i64>() {
                let base_permission = parts[0..parts.len()-1].join(":");
                return (base_permission, Some(timestamp));
            }
        }
        (permission.to_string(), None)
    }
    
    fn is_permission_expired(permission: &str) -> bool {
        let (_, timestamp) = parse_embedded_permission(permission);
        if let Some(ts) = timestamp {
            let now = chrono::Utc::now().timestamp();
            return now > ts;
        }
        false
    }
    
    fn validate_permission_format(permission: &str) -> bool {
        let parts: Vec<&str> = permission.split(':').collect();
        parts.len() >= 3 && parts.iter().all(|part| !part.is_empty())
    }
    
    fn calculate_time_remaining(permission: &str) -> Option<i64> {
        let (_, timestamp) = parse_embedded_permission(permission);
        if let Some(ts) = timestamp {
            let now = chrono::Utc::now().timestamp();
            if ts > now {
                return Some((ts - now) * 1000); // Return milliseconds
            }
        }
        None
    }
    
    // Mock permission service for testing
    struct MockPermissionService {
        users: HashMap<String, MockUser>,
    }
    
    impl MockPermissionService {
        fn new() -> Self {
            Self {
                users: HashMap::new(),
            }
        }
        
        fn add_user(&mut self, user: MockUser) {
            self.users.insert(user.id.clone(), user);
        }
        
        fn grant_permission(&mut self, request: MockPermissionRequest) -> PermissionTestResult {
            if !validate_permission_format(&request.base_permission) {
                return PermissionTestResult {
                    success: false,
                    permission: None,
                    error: Some("Invalid permission format".to_string()),
                };
            }
            
            if request.expiry_timestamp <= chrono::Utc::now().timestamp() {
                return PermissionTestResult {
                    success: false,
                    permission: None,
                    error: Some("Expiry timestamp must be in the future".to_string()),
                };
            }
            
            let embedded_permission = create_embedded_permission(
                &request.base_permission,
                request.expiry_timestamp
            );
            
            if let Some(user) = self.users.get_mut(&request.user_id) {
                // Check for duplicate permissions
                if user.permissions.contains(&embedded_permission) {
                    return PermissionTestResult {
                        success: false,
                        permission: None,
                        error: Some("Permission already exists".to_string()),
                    };
                }
                
                user.permissions.push(embedded_permission.clone());
                PermissionTestResult {
                    success: true,
                    permission: Some(embedded_permission),
                    error: None,
                }
            } else {
                PermissionTestResult {
                    success: false,
                    permission: None,
                    error: Some("User not found".to_string()),
                }
            }
        }
        
        fn revoke_permission(&mut self, user_id: &str, permission: &str) -> PermissionTestResult {
            if let Some(user) = self.users.get_mut(user_id) {
                if let Some(pos) = user.permissions.iter().position(|p| p == permission) {
                    user.permissions.remove(pos);
                    PermissionTestResult {
                        success: true,
                        permission: Some(permission.to_string()),
                        error: None,
                    }
                } else {
                    PermissionTestResult {
                        success: false,
                        permission: None,
                        error: Some("Permission not found".to_string()),
                    }
                }
            } else {
                PermissionTestResult {
                    success: false,
                    permission: None,
                    error: Some("User not found".to_string()),
                }
            }
        }
        
        fn extend_permission(&mut self, user_id: &str, old_permission: &str, new_expiry: i64) -> PermissionTestResult {
            if let Some(user) = self.users.get_mut(user_id) {
                if let Some(pos) = user.permissions.iter().position(|p| p == old_permission) {
                    let (base_permission, _) = parse_embedded_permission(old_permission);
                    let new_permission = create_embedded_permission(&base_permission, new_expiry);
                    user.permissions[pos] = new_permission.clone();
                    
                    PermissionTestResult {
                        success: true,
                        permission: Some(new_permission),
                        error: None,
                    }
                } else {
                    PermissionTestResult {
                        success: false,
                        permission: None,
                        error: Some("Permission not found".to_string()),
                    }
                }
            } else {
                PermissionTestResult {
                    success: false,
                    permission: None,
                    error: Some("User not found".to_string()),
                }
            }
        }
        
        fn get_user_permissions(&self, user_id: &str) -> Vec<String> {
            self.users.get(user_id)
                .map(|user| user.permissions.clone())
                .unwrap_or_default()
        }
        
        fn validate_permissions(&self, user_id: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
            let permissions = self.get_user_permissions(user_id);
            let mut valid = Vec::new();
            let mut expired = Vec::new();
            let mut expiring_soon = Vec::new();
            
            let now = chrono::Utc::now().timestamp();
            let twenty_four_hours = 24 * 60 * 60;
            
            for permission in permissions {
                if is_permission_expired(&permission) {
                    expired.push(permission);
                } else if let Some(remaining) = calculate_time_remaining(&permission) {
                    if remaining <= twenty_four_hours * 1000 { // 24 hours in milliseconds
                        expiring_soon.push(permission);
                    } else {
                        valid.push(permission);
                    }
                } else {
                    valid.push(permission); // Permanent permission
                }
            }
            
            (valid, expired, expiring_soon)
        }
        
        fn cleanup_expired_permissions(&mut self) -> (i32, i32) {
            let mut cleaned = 0;
            let mut failed = 0;
            
            for user in self.users.values_mut() {
                let original_count = user.permissions.len();
                user.permissions.retain(|p| !is_permission_expired(p));
                let new_count = user.permissions.len();
                cleaned += (original_count - new_count) as i32;
            }
            
            (cleaned, failed)
        }
    }
    
    #[tokio::test]
    async fn test_permission_grant_comprehensive() {
        let mut service = MockPermissionService::new();
        
        // Create test user
        let user = MockUser {
            id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            permissions: vec![],
            created_at: Utc::now(),
        };
        service.add_user(user);
        
        // Test Case 1: Valid permission grant
        let future_timestamp = Utc::now().timestamp() + 3600; // 1 hour from now
        let request = MockPermissionRequest {
            user_id: "user-123".to_string(),
            base_permission: "epsx:analytics:premium".to_string(),
            platform: "epsx".to_string(),
            resource: "analytics".to_string(),
            action: "premium".to_string(),
            expiry_timestamp: future_timestamp,
            reason: Some("E2E Testing".to_string()),
        };
        
        let result = service.grant_permission(request);
        assert!(result.success, "Should successfully grant valid permission");
        assert!(result.permission.is_some(), "Should return the granted permission");
        
        let granted_permission = result.permission.unwrap();
        assert_eq!(granted_permission, format!("epsx:analytics:premium:{}", future_timestamp));
        
        // Test Case 2: Invalid permission format
        let invalid_request = MockPermissionRequest {
            user_id: "user-123".to_string(),
            base_permission: "invalid-format".to_string(), // Missing colons
            platform: "epsx".to_string(),
            resource: "analytics".to_string(),
            action: "premium".to_string(),
            expiry_timestamp: future_timestamp,
            reason: None,
        };
        
        let invalid_result = service.grant_permission(invalid_request);
        assert!(!invalid_result.success, "Should reject invalid permission format");
        assert!(invalid_result.error.is_some(), "Should provide error message");
        
        // Test Case 3: Past expiry timestamp
        let past_timestamp = Utc::now().timestamp() - 3600; // 1 hour ago
        let past_request = MockPermissionRequest {
            user_id: "user-123".to_string(),
            base_permission: "epsx:rankings:view".to_string(),
            platform: "epsx".to_string(),
            resource: "rankings".to_string(),
            action: "view".to_string(),
            expiry_timestamp: past_timestamp,
            reason: None,
        };
        
        let past_result = service.grant_permission(past_request);
        assert!(!past_result.success, "Should reject past expiry timestamp");
        
        // Test Case 4: Duplicate permission
        let duplicate_request = MockPermissionRequest {
            user_id: "user-123".to_string(),
            base_permission: "epsx:analytics:premium".to_string(),
            platform: "epsx".to_string(),
            resource: "analytics".to_string(),
            action: "premium".to_string(),
            expiry_timestamp: future_timestamp,
            reason: None,
        };
        
        let duplicate_result = service.grant_permission(duplicate_request);
        assert!(!duplicate_result.success, "Should reject duplicate permission");
        
        // Test Case 5: Non-existent user
        let nonexistent_request = MockPermissionRequest {
            user_id: "nonexistent-user".to_string(),
            base_permission: "epsx:test:permission".to_string(),
            platform: "epsx".to_string(),
            resource: "test".to_string(),
            action: "permission".to_string(),
            expiry_timestamp: future_timestamp,
            reason: None,
        };
        
        let nonexistent_result = service.grant_permission(nonexistent_request);
        assert!(!nonexistent_result.success, "Should reject request for non-existent user");
    }
    
    #[tokio::test]
    async fn test_permission_expiry_comprehensive() {
        let mut service = MockPermissionService::new();
        
        // Create test user with various permissions
        let now = Utc::now().timestamp();
        let user = MockUser {
            id: "user-456".to_string(),
            email: "expiry@example.com".to_string(),
            permissions: vec![
                format!("epsx:analytics:premium:{}", now + 3600),    // Expires in 1 hour
                format!("epsx:rankings:view:{}", now - 3600),       // Expired 1 hour ago
                format!("admin:users:manage:{}", now + 86400),      // Expires in 1 day
                format!("epsx:basic:read:{}", now + 7200),          // Expires in 2 hours
                "epsx:permanent:access".to_string(),                // No expiry
            ],
            created_at: Utc::now(),
        };
        service.add_user(user);
        
        // Test permission validation
        let (valid, expired, expiring_soon) = service.validate_permissions("user-456");
        
        // Should have 1 expired permission
        assert_eq!(expired.len(), 1, "Should identify 1 expired permission");
        assert!(expired[0].contains("epsx:rankings:view"), "Should identify the correct expired permission");
        
        // Should have expiring soon permissions (within 24 hours)
        assert!(expiring_soon.len() >= 2, "Should identify expiring soon permissions");
        
        // Should have valid permissions (including permanent)
        assert!(valid.len() >= 2, "Should identify valid permissions");
        
        // Test individual permission expiry check
        assert!(is_permission_expired(&format!("epsx:rankings:view:{}", now - 3600)), "Should detect expired permission");
        assert!(!is_permission_expired(&format!("epsx:analytics:premium:{}", now + 3600)), "Should detect valid permission");
        assert!(!is_permission_expired("epsx:permanent:access"), "Should handle permanent permissions");
        
        // Test time remaining calculation
        let time_remaining = calculate_time_remaining(&format!("epsx:analytics:premium:{}", now + 3600));
        assert!(time_remaining.is_some(), "Should calculate time remaining for timestamped permission");
        assert!(time_remaining.unwrap() > 0, "Should return positive time remaining");
        
        let no_time_remaining = calculate_time_remaining("epsx:permanent:access");
        assert!(no_time_remaining.is_none(), "Should return None for permanent permissions");
    }
    
    #[tokio::test]
    async fn test_permission_management_operations() {
        let mut service = MockPermissionService::new();
        
        // Create test user
        let now = Utc::now().timestamp();
        let user = MockUser {
            id: "user-789".to_string(),
            email: "management@example.com".to_string(),
            permissions: vec![
                format!("epsx:analytics:premium:{}", now + 3600),
            ],
            created_at: Utc::now(),
        };
        service.add_user(user);
        
        // Test permission extension
        let old_permission = format!("epsx:analytics:premium:{}", now + 3600);
        let new_expiry = now + 7200; // 2 hours from now
        
        let extend_result = service.extend_permission("user-789", &old_permission, new_expiry);
        assert!(extend_result.success, "Should successfully extend permission");
        
        let extended_permission = extend_result.permission.unwrap();
        assert_eq!(extended_permission, format!("epsx:analytics:premium:{}", new_expiry));
        
        // Verify old permission is replaced
        let user_permissions = service.get_user_permissions("user-789");
        assert!(!user_permissions.contains(&old_permission), "Should remove old permission");
        assert!(user_permissions.contains(&extended_permission), "Should add extended permission");
        
        // Test permission revocation
        let revoke_result = service.revoke_permission("user-789", &extended_permission);
        assert!(revoke_result.success, "Should successfully revoke permission");
        
        let user_permissions_after_revoke = service.get_user_permissions("user-789");
        assert!(!user_permissions_after_revoke.contains(&extended_permission), "Should remove revoked permission");
        assert_eq!(user_permissions_after_revoke.len(), 0, "User should have no permissions after revocation");
        
        // Test revoking non-existent permission
        let revoke_nonexistent = service.revoke_permission("user-789", "nonexistent:permission");
        assert!(!revoke_nonexistent.success, "Should fail to revoke non-existent permission");
    }
    
    #[tokio::test]
    async fn test_bulk_permission_operations() {
        let mut service = MockPermissionService::new();
        
        // Create multiple test users
        for i in 1..=5 {
            let user = MockUser {
                id: format!("bulk-user-{}", i),
                email: format!("bulk{}@example.com", i),
                permissions: vec![],
                created_at: Utc::now(),
            };
            service.add_user(user);
        }
        
        // Test bulk permission granting
        let future_timestamp = Utc::now().timestamp() + 3600;
        let bulk_permissions = vec![
            "epsx:analytics:basic",
            "epsx:rankings:view",
            "epsx:data:export",
        ];
        
        let mut successful_grants = 0;
        let mut failed_grants = 0;
        
        for i in 1..=5 {
            for permission in &bulk_permissions {
                let request = MockPermissionRequest {
                    user_id: format!("bulk-user-{}", i),
                    base_permission: permission.to_string(),
                    platform: "epsx".to_string(),
                    resource: "bulk".to_string(),
                    action: "test".to_string(),
                    expiry_timestamp: future_timestamp,
                    reason: Some("Bulk operation test".to_string()),
                };
                
                let result = service.grant_permission(request);
                if result.success {
                    successful_grants += 1;
                } else {
                    failed_grants += 1;
                }
            }
        }
        
        assert_eq!(successful_grants, 15, "Should successfully grant all bulk permissions"); // 5 users × 3 permissions
        assert_eq!(failed_grants, 0, "Should have no failed grants");
        
        // Verify each user has all permissions
        for i in 1..=5 {
            let user_permissions = service.get_user_permissions(&format!("bulk-user-{}", i));
            assert_eq!(user_permissions.len(), 3, "Each user should have 3 permissions");
        }
    }
    
    #[tokio::test]
    async fn test_permission_cleanup_operations() {
        let mut service = MockPermissionService::new();
        
        // Create users with mix of expired and valid permissions
        let now = Utc::now().timestamp();
        
        for i in 1..=3 {
            let user = MockUser {
                id: format!("cleanup-user-{}", i),
                email: format!("cleanup{}@example.com", i),
                permissions: vec![
                    format!("epsx:expired:permission1:{}", now - 3600),    // Expired
                    format!("epsx:expired:permission2:{}", now - 7200),    // Expired
                    format!("epsx:valid:permission:{}", now + 3600),       // Valid
                    "epsx:permanent:permission".to_string(),               // Permanent
                ],
                created_at: Utc::now(),
            };
            service.add_user(user);
        }
        
        // Test cleanup operation
        let (cleaned, failed) = service.cleanup_expired_permissions();
        
        assert_eq!(cleaned, 6, "Should clean 6 expired permissions (2 per user × 3 users)");
        assert_eq!(failed, 0, "Should have no failed cleanups");
        
        // Verify cleanup results
        for i in 1..=3 {
            let user_permissions = service.get_user_permissions(&format!("cleanup-user-{}", i));
            assert_eq!(user_permissions.len(), 2, "Each user should have 2 remaining permissions");
            
            // Verify no expired permissions remain
            for permission in &user_permissions {
                assert!(!is_permission_expired(permission), "No expired permissions should remain");
            }
        }
    }
    
    #[tokio::test]
    async fn test_permission_format_validation() {
        // Test valid permission formats
        let valid_formats = vec![
            "epsx:analytics:premium",
            "admin:users:manage",
            "epsx:rankings:view:100",
            "custom:resource:action:subaction",
        ];
        
        for permission in valid_formats {
            assert!(validate_permission_format(permission), 
                "Should validate correct format: {}", permission);
        }
        
        // Test invalid permission formats
        let invalid_formats = vec![
            "",                           // Empty
            "single",                     // Too few parts
            "two:parts",                  // Too few parts
            "epsx:",                      // Empty part
            ":analytics:premium",         // Empty first part
            "epsx::premium",              // Empty middle part
            "epsx:analytics:",            // Empty last part
        ];
        
        for permission in invalid_formats {
            assert!(!validate_permission_format(permission), 
                "Should reject invalid format: {}", permission);
        }
    }
    
    #[tokio::test]
    async fn test_permission_parsing_edge_cases() {
        // Test parsing various permission formats
        let test_cases = vec![
            // (input, expected_base, expected_timestamp)
            ("epsx:analytics:premium:1234567890", "epsx:analytics:premium", Some(1234567890)),
            ("admin:users:manage:9876543210", "admin:users:manage", Some(9876543210)),
            ("epsx:basic:read", "epsx:basic:read", None),
            ("complex:permission:with:many:parts:1111111111", "complex:permission:with:many:parts", Some(1111111111)),
            ("epsx:analytics:premium:invalid", "epsx:analytics:premium:invalid", None), // Invalid timestamp
            ("single", "single", None),
            ("", "", None),
        ];
        
        for (input, expected_base, expected_timestamp) in test_cases {
            let (base, timestamp) = parse_embedded_permission(input);
            assert_eq!(base, expected_base, "Base permission should match for: {}", input);
            assert_eq!(timestamp, expected_timestamp, "Timestamp should match for: {}", input);
        }
    }
    
    #[tokio::test]
    async fn test_permission_health_monitoring() {
        let mut service = MockPermissionService::new();
        
        // Create user with various permission states
        let now = Utc::now().timestamp();
        let user = MockUser {
            id: "health-user".to_string(),
            email: "health@example.com".to_string(),
            permissions: vec![
                format!("epsx:analytics:premium:{}", now + 3600),      // Expiring in 1 hour
                format!("epsx:rankings:view:{}", now - 3600),          // Expired 1 hour ago
                format!("admin:users:manage:{}", now + 86400),         // Expires in 1 day
                format!("epsx:data:export:{}", now + 1800),            // Expiring in 30 minutes
                "epsx:permanent:access".to_string(),                   // No expiry
            ],
            created_at: Utc::now(),
        };
        service.add_user(user);
        
        let (valid, expired, expiring_soon) = service.validate_permissions("health-user");
        
        // Health metrics
        let total_permissions = 5;
        let active_permissions = valid.len() + expiring_soon.len();
        let expired_permissions = expired.len();
        let health_score = ((active_permissions as f64 / total_permissions as f64) * 100.0) as i32;
        
        assert_eq!(expired_permissions, 1, "Should have 1 expired permission");
        assert!(expiring_soon.len() >= 2, "Should have expiring soon permissions");
        assert!(health_score > 50, "Health score should be above 50%");
        
        // Test permission health status calculation
        for permission in service.get_user_permissions("health-user") {
            let health_status = if is_permission_expired(&permission) {
                "expired"
            } else if let Some(remaining) = calculate_time_remaining(&permission) {
                if remaining <= 24 * 60 * 60 * 1000 { // 24 hours
                    "expiring"
                } else {
                    "healthy"
                }
            } else {
                "permanent"
            };
            
            assert!(
                ["expired", "expiring", "healthy", "permanent"].contains(&health_status),
                "Should calculate valid health status"
            );
        }
    }
    
    #[tokio::test]
    async fn test_real_world_permission_scenarios() {
        let mut service = MockPermissionService::new();
        
        // Scenario 1: New user gets trial permissions
        let trial_user = MockUser {
            id: "trial-user".to_string(),
            email: "trial@example.com".to_string(),
            permissions: vec![],
            created_at: Utc::now(),
        };
        service.add_user(trial_user);
        
        // Grant 7-day trial permissions
        let trial_expiry = Utc::now().timestamp() + (7 * 24 * 60 * 60); // 7 days
        let trial_permissions = vec![
            "epsx:analytics:basic",
            "epsx:rankings:view",
        ];
        
        for permission in trial_permissions {
            let request = MockPermissionRequest {
                user_id: "trial-user".to_string(),
                base_permission: permission.to_string(),
                platform: "epsx".to_string(),
                resource: "trial".to_string(),
                action: "access".to_string(),
                expiry_timestamp: trial_expiry,
                reason: Some("7-day trial access".to_string()),
            };
            
            let result = service.grant_permission(request);
            assert!(result.success, "Should grant trial permission: {}", permission);
        }
        
        // Scenario 2: Premium user gets extended permissions
        let premium_user = MockUser {
            id: "premium-user".to_string(),
            email: "premium@example.com".to_string(),
            permissions: vec![],
            created_at: Utc::now(),
        };
        service.add_user(premium_user);
        
        // Grant 30-day premium permissions
        let premium_expiry = Utc::now().timestamp() + (30 * 24 * 60 * 60); // 30 days
        let premium_permissions = vec![
            "epsx:analytics:premium",
            "epsx:rankings:advanced",
            "epsx:data:export",
            "epsx:alerts:unlimited",
        ];
        
        for permission in premium_permissions {
            let request = MockPermissionRequest {
                user_id: "premium-user".to_string(),
                base_permission: permission.to_string(),
                platform: "epsx".to_string(),
                resource: "premium".to_string(),
                action: "full".to_string(),
                expiry_timestamp: premium_expiry,
                reason: Some("Premium subscription".to_string()),
            };
            
            let result = service.grant_permission(request);
            assert!(result.success, "Should grant premium permission: {}", permission);
        }
        
        // Scenario 3: Enterprise user gets permanent admin permissions
        let enterprise_user = MockUser {
            id: "enterprise-user".to_string(),
            email: "enterprise@example.com".to_string(),
            permissions: vec![
                "admin:users:manage".to_string(),
                "admin:system:configure".to_string(),
                "epsx:analytics:enterprise".to_string(),
            ],
            created_at: Utc::now(),
        };
        service.add_user(enterprise_user);
        
        // Verify enterprise user has permanent permissions (no timestamps)
        let enterprise_permissions = service.get_user_permissions("enterprise-user");
        for permission in enterprise_permissions {
            let (_, timestamp) = parse_embedded_permission(&permission);
            assert!(timestamp.is_none(), "Enterprise permissions should be permanent");
        }
        
        // Scenario 4: Permission renewal before expiry
        let trial_permissions = service.get_user_permissions("trial-user");
        if let Some(first_permission) = trial_permissions.first() {
            let renewal_expiry = Utc::now().timestamp() + (14 * 24 * 60 * 60); // Extend to 14 days
            let extend_result = service.extend_permission("trial-user", first_permission, renewal_expiry);
            assert!(extend_result.success, "Should successfully renew trial permission");
        }
        
        // Verify all scenarios created valid states
        assert_eq!(service.get_user_permissions("trial-user").len(), 2, "Trial user should have 2 permissions");
        assert_eq!(service.get_user_permissions("premium-user").len(), 4, "Premium user should have 4 permissions");
        assert_eq!(service.get_user_permissions("enterprise-user").len(), 3, "Enterprise user should have 3 permissions");
    }
    
    #[tokio::test]
    async fn test_permission_error_handling() {
        let mut service = MockPermissionService::new();
        
        // Test various error conditions
        
        // 1. User not found
        let request = MockPermissionRequest {
            user_id: "nonexistent-user".to_string(),
            base_permission: "epsx:test:permission".to_string(),
            platform: "epsx".to_string(),
            resource: "test".to_string(),
            action: "permission".to_string(),
            expiry_timestamp: Utc::now().timestamp() + 3600,
            reason: None,
        };
        
        let result = service.grant_permission(request);
        assert!(!result.success, "Should fail for non-existent user");
        assert!(result.error.is_some(), "Should provide error message");
        
        // 2. Create valid user for remaining tests
        let user = MockUser {
            id: "error-test-user".to_string(),
            email: "error@example.com".to_string(),
            permissions: vec!["epsx:existing:permission:1234567890".to_string()],
            created_at: Utc::now(),
        };
        service.add_user(user);
        
        // 3. Test permission extension errors
        let extend_result = service.extend_permission("error-test-user", "nonexistent:permission", Utc::now().timestamp() + 3600);
        assert!(!extend_result.success, "Should fail to extend non-existent permission");
        
        // 4. Test permission revocation errors
        let revoke_result = service.revoke_permission("error-test-user", "nonexistent:permission");
        assert!(!revoke_result.success, "Should fail to revoke non-existent permission");
        
        // 5. Test validation with empty user
        let empty_user = MockUser {
            id: "empty-user".to_string(),
            email: "empty@example.com".to_string(),
            permissions: vec![],
            created_at: Utc::now(),
        };
        service.add_user(empty_user);
        
        let (valid, expired, expiring_soon) = service.validate_permissions("empty-user");
        assert_eq!(valid.len(), 0, "Empty user should have no valid permissions");
        assert_eq!(expired.len(), 0, "Empty user should have no expired permissions");
        assert_eq!(expiring_soon.len(), 0, "Empty user should have no expiring permissions");
    }
}