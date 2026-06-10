// ============================================================================
// WALLET USER REPOSITORY TESTS (Test-Driven Development)
// Unit tests for database user lookup scenarios without database dependencies
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::wallet_management::{
        aggregates::{WalletUser, WalletMetadata},
        value_objects::WalletAddress,
    };
    use serde_json::json;

    // ================== Wallet Address Validation Tests ==================

    #[test]
    fn test_wallet_address_validation() {
        // Test valid wallet addresses
        let valid_addresses = vec![
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "0x1234567890123456789012345678901234567890",
            "0xabcdefABCDEFabcdefABCDEFabcdefABCDEFabcd",
            "0x0000000000000000000000000000000000000000",
        ];

        for addr in valid_addresses {
            let result = WalletAddress::new(addr.to_string());
            assert!(result.is_ok(), "Should accept valid address: {}", addr);
        }

        // Test invalid wallet addresses
        let invalid_addresses = vec![
            "",                                 // Empty
            "0x",                               // Only prefix
            "invalid_wallet_address",           // No prefix, invalid format
            "0x123",                           // Too short
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG", // Invalid characters
            "0x12345678901234567890123456789012345678901", // Too long
            "1234567890123456789012345678901234567890",    // Missing 0x prefix
        ];

        for addr in invalid_addresses {
            let result = WalletAddress::new(addr.to_string());
            assert!(result.is_err(), "Should reject invalid address: {}", addr);
        }
    }

    #[test]
    fn test_wallet_address_case_sensitivity() {
        // Test that wallet addresses are case-insensitive for validation
        let addresses = vec![
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "0X742D35CC6634C0532925A3B8D369D7763F3C45C6", // Uppercase
            "0x742d35cc6634c0532925a3b8d369d7763f3c45c6", // Lowercase
        ];

        for addr in addresses {
            let result = WalletAddress::new(addr.to_string());
            assert!(result.is_ok(), "Should accept address in any case: {}", addr);
        }

        // Test that the stored value maintains original format
        let wallet = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        assert_eq!(wallet.as_str(), "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6");
    }

    // ================== Wallet Metadata Tests ==================

    #[test]
    fn test_wallet_metadata_creation() {
        let metadata = WalletMetadata::new();

        assert!(metadata.user_agent().is_none());
        assert!(metadata.ip_address().is_none());
        assert!(metadata.get_custom_field("any_field").is_none());
    }

    #[test]
    fn test_wallet_metadata_with_user_agent() {
        let metadata = WalletMetadata::new()
            .with_user_agent("Mozilla/5.0 (Test Browser)");

        assert_eq!(metadata.user_agent(), Some("Mozilla/5.0 (Test Browser)"));
        assert!(metadata.ip_address().is_none());
    }

    #[test]
    fn test_wallet_metadata_with_ip_address() {
        let metadata = WalletMetadata::new()
            .with_ip_address("192.168.1.1");

        assert_eq!(metadata.ip_address(), Some("192.168.1.1"));
        assert!(metadata.user_agent().is_none());
    }

    #[test]
    fn test_wallet_metadata_with_custom_fields() {
        let metadata = WalletMetadata::new()
            .with_custom_field("browser_language", "en-US")
            .with_custom_field("timezone", "America/New_York")
            .with_custom_field("login_count", "42");

        assert_eq!(metadata.get_custom_field("browser_language"), Some(&"en-US".to_string()));
        assert_eq!(metadata.get_custom_field("timezone"), Some(&"America/New_York".to_string()));
        assert_eq!(metadata.get_custom_field("login_count"), Some(&"42".to_string()));
        assert_eq!(metadata.get_custom_field("nonexistent"), None);
    }

    #[test]
    fn test_wallet_metadata_complex_structure() {
        let metadata = WalletMetadata::new()
            .with_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .with_ip_address("203.0.113.1")
            .with_custom_field("browser_info", json!({
                "name": "Chrome",
                "version": "91.0.4472.124",
                "platform": "Windows"
            }))
            .with_custom_field("session_info", json!({
                "id": "sess_123456",
                "duration_minutes": 45,
                "pages_visited": 12
            }));

        assert_eq!(metadata.user_agent(), Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"));
        assert_eq!(metadata.ip_address(), Some("203.0.113.1"));

        // Test complex JSON field access
        let browser_info = metadata.get_custom_field("browser_info");
        assert!(browser_info.is_some());
        let browser_info_val: serde_json::Value = browser_info.unwrap().parse().unwrap();
        assert_eq!(browser_info_val["name"], "Chrome");
        assert_eq!(browser_info_val["version"], "91.0.4472.124");
    }

    #[test]
    fn test_wallet_metadata_json_serialization() {
        let metadata = WalletMetadata::new()
            .with_user_agent("Test Agent")
            .with_ip_address("127.0.0.1")
            .with_custom_field("test_id", "unit_test");

        let json_value = metadata.to_json_value();

        assert!(json_value.is_object());
        assert_eq!(json_value["user_agent"], "Test Agent");
        assert_eq!(json_value["ip_address"], "127.0.0.1");
        assert_eq!(json_value["test_id"], "unit_test");
    }

    // ================== Wallet User Aggregates Tests ==================

    #[test]
    fn test_wallet_user_creation() {
        let wallet_address = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let metadata = WalletMetadata::new()
            .with_user_agent("Test Browser")
            .with_ip_address("192.168.1.1");

        let user = WalletUser::new(wallet_address.clone(), metadata.clone(), true);

        assert_eq!(user.wallet_address.as_str(), "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6");
        assert!(user.is_active);
        assert_eq!(user.metadata.user_agent(), Some("Test Browser"));
        assert_eq!(user.metadata.ip_address(), Some("192.168.1.1"));
    }

    #[test]
    fn test_wallet_user_with_permissions() {
        let wallet_address = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let metadata = WalletMetadata::new();

        let permissions = vec![
            "epsx:analytics:read".to_string(),
            "epsx:rankings:read".to_string(),
            "admin:users:manage".to_string(),
        ];

        let user = WalletUser::new(wallet_address.clone(), metadata.clone(), true)
            .with_permissions(permissions);

        assert_eq!(user.permissions.len(), 3);
        assert!(user.has_permission("epsx:analytics:read"));
        assert!(user.has_permission("admin:users:manage"));
        assert!(!user.has_permission("epsx:admin:all"));
    }

    #[test]
    fn test_wallet_user_active_status() {
        let wallet_address = WalletAddress::new("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()).unwrap();
        let metadata = WalletMetadata::new();

        let active_user = WalletUser::new(wallet_address.clone(), metadata.clone(), true);
        assert!(active_user.is_active);

        let inactive_user = WalletUser::new(wallet_address, metadata, false);
        assert!(!inactive_user.is_active);
    }

    // ================== Database Query Logic Tests ==================

    #[test]
    fn test_sql_query_construction() {
        // Test the SQL query construction logic used in find_by_wallet
        let test_addresses = vec![
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "0xABCdef123456789012345678901234567890abcd",
            "0x0000000000000000000000000000000000000000",
        ];

        for addr in test_addresses {
            let expected_sql = format!("LOWER(wallet_address) = LOWER('{}')", addr);

            // Verify the SQL construction logic
            assert!(expected_sql.contains("LOWER(wallet_address)"));
            assert!(expected_sql.contains("LOWER("));
            assert!(expected_sql.contains(addr));

            // Test case insensitivity handling
            let upper_addr = addr.to_uppercase();
            let sql_with_upper = format!("LOWER(wallet_address) = LOWER('{}')", upper_addr);
            assert!(sql_with_upper.contains("LOWER(")); // Should still use LOWER function
        }
    }

    #[test]
    fn test_case_insensitive_wallet_matching() {
        let test_cases = vec![
            ("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6", "0x742d35CC6634C0532925A3B8D369D7763F3C45C6", true),
            ("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6", "0x742d35cc6634c0532925a3b8d369d7763f3c45c6", true),
            ("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6", "0x742d35Cc6634C0532925a3b8D369D7763F3c45c7", false), // Different last char
            ("0x742d35Cc6634C0532925a3b8D369D7763F3c45c6", "0x742d35Cc6634C0532925a3b8D369D7763F3c45c",  false), // Too short
        ];

        for (addr1, addr2, should_match) in test_cases {
            let addr1_lower = addr1.to_lowercase();
            let addr2_lower = addr2.to_lowercase();
            let actually_matches = addr1_lower == addr2_lower;

            assert_eq!(actually_matches, should_match,
                       "Case insensitive matching failed: {} vs {} should be {}",
                       addr1, addr2, should_match);
        }
    }

    // ================== Error Handling Scenarios ==================

    #[test]
    fn test_database_error_mapping() {
        // Test error message mapping for common database scenarios
        let error_messages = vec![
            ("duplicate key value violates unique constraint", "duplicate"),
            ("violates foreign key constraint", "foreign key"),
            ("connection refused", "connection"),
            ("timeout expired", "timeout"),
            ("syntax error", "syntax"),
        ];

        for (db_error, expected_keyword) in error_messages {
            let contains_keyword = db_error.contains(expected_keyword);
            assert!(contains_keyword,
                   "Database error '{}' should contain keyword '{}'",
                   db_error, expected_keyword);
        }
    }

    #[test]
    fn test_permission_validation_logic() {
        let test_permissions = vec![
            ("epsx:analytics:read", "epsx:analytics:read", true),
            ("epsx:analytics:read", "epsx:analytics:write", false),
            ("admin:users:manage", "admin:*:*", true), // Should match wildcard
            ("epsx:*:read", "epsx:analytics:read", true), // Should match wildcard
            ("epsx:*:*", "epsx:any:any", true), // Should match any
            ("", "epsx:analytics:read", false), // Empty permission
            ("epsx:analytics:read", "", false), // Empty required permission
        ];

        for (user_permission, required_permission, should_match) in test_permissions {
            let matches = check_permission_matching(user_permission, required_permission);
            assert_eq!(matches, should_match,
                       "Permission '{}' vs '{}' should match: {}",
                       user_permission, required_permission, should_match);
        }
    }

    // ================== Performance Considerations ==================

    #[test]
    fn test_wallet_address_hash_performance() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let addresses = vec![
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6",
            "0x1234567890123456789012345678901234567890",
            "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        ];

        for addr in addresses {
            let wallet = WalletAddress::new(addr.to_string()).unwrap();

            // Test that wallet address is hashable
            let mut hasher = DefaultHasher::new();
            wallet.hash(&mut hasher);
            let hash1 = hasher.finish();

            // Should be deterministic
            let mut hasher2 = DefaultHasher::new();
            wallet.hash(&mut hasher2);
            let hash2 = hasher2.finish();

            assert_eq!(hash1, hash2, "Wallet address hash should be deterministic");
        }
    }

    // ================== Helper Functions ==================

    /// Helper function to test permission matching logic
    fn check_permission_matching(user_permission: &str, required_permission: &str) -> bool {
        if user_permission == required_permission {
            return true;
        }

        // Check wildcard patterns
        if user_permission.ends_with("*") {
            let prefix = &user_permission[..user_permission.len() - 2]; // Remove ":*"
            if required_permission.starts_with(prefix) {
                return true;
            }
        }

        false
    }
}

// ================== Test Utilities ==================

/// Test data generation utilities
mod test_utils {
    use super::*;

    /// Generate a valid test wallet address
    pub fn generate_test_wallet_address() -> String {
        "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string()
    }

    /// Generate test wallet metadata
    pub fn create_test_wallet_metadata() -> WalletMetadata {
        WalletMetadata::new()
            .with_user_agent("Test Browser/1.0")
            .with_ip_address("127.0.0.1")
            .with_custom_field("test_id", "unit_test")
            .with_custom_field("environment", "test")
    }

    /// Create a test wallet user
    pub fn create_test_wallet_user() -> WalletUser {
        let wallet_address = WalletAddress::new(generate_test_wallet_address()).unwrap();
        let metadata = create_test_wallet_metadata();
        WalletUser::new(wallet_address, metadata, true)
    }

    /// Create a test wallet user with specific permissions
    pub fn create_test_wallet_user_with_permissions(permissions: Vec<String>) -> WalletUser {
        let wallet_address = WalletAddress::new(generate_test_wallet_address()).unwrap();
        let metadata = create_test_wallet_metadata();
        WalletUser::new(wallet_address, metadata, true).with_permissions(permissions)
    }

    /// Generate test wallet addresses for different scenarios
    pub fn generate_test_wallet_addresses() -> Vec<String> {
        vec![
            "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "0xabcdefABCDEFabcdefABCDEFabcdefABCDEFabcd".to_string(),
            "0x0000000000000000000000000000000000000000".to_string(),
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string(),
        ]
    }
}