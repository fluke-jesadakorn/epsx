//! Test Timestamp Utilities
//! 
//! Provides consistent timestamp values for testing embedded permissions
//! and other time-sensitive functionality. Replaces hardcoded magic numbers
//! with semantic, reusable constants.

use std::time::{SystemTime, UNIX_EPOCH};

/// Common test timestamps with semantic names
pub struct TestTimestamps;

impl TestTimestamps {
    /// January 1, 2024 00:00:00 UTC (1704067200)
    pub const YEAR_2024_START: u64 = 1704067200;
    
    /// December 31, 2024 23:59:59 UTC (1735689599) 
    pub const YEAR_2024_END: u64 = 1735689599;
    
    /// June 1, 2024 00:00:00 UTC (1717200000) - Mid-year timestamp
    pub const MID_2024: u64 = 1717200000;
    
    /// Legacy test timestamp (Dec 30, 2023) - for backward compatibility
    pub const LEGACY_TEST: u64 = 1703980800;
    
    /// Far future timestamp (Jan 1, 2030) for long-lived permissions
    pub const FAR_FUTURE: u64 = 1893456000;
    
    /// Near future (30 days from now) - calculated dynamically
    pub fn thirty_days_from_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + (30 * 24 * 60 * 60)
    }
    
    /// One hour from now - for short-lived test permissions
    pub fn one_hour_from_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + (60 * 60)
    }
    
    /// One hour ago - for expired permissions testing
    pub fn one_hour_ago() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (60 * 60)
    }
    
    /// Get current timestamp
    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Permission template builder for consistent test data
pub struct PermissionBuilder;

impl PermissionBuilder {
    /// Create analytics permission with timestamp
    pub fn analytics_with_expiry(timestamp: u64) -> String {
        format!("epsx:analytics:view:{}", timestamp)
    }
    
    /// Create admin permission with timestamp
    pub fn admin_with_expiry(timestamp: u64) -> String {
        format!("admin:users:manage:{}", timestamp)
    }
    
    /// Create rankings permission with limit and timestamp
    pub fn rankings_with_limit_and_expiry(limit: u32, timestamp: u64) -> String {
        format!("epsx:rankings:view:{}:{}", limit, timestamp)
    }
    
    /// Create payment permission with timestamp
    pub fn payments_with_expiry(timestamp: u64) -> String {
        format!("epsx-pay:transactions:read:{}", timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_constants() {
        // Verify constants are reasonable
        assert!(TestTimestamps::YEAR_2024_START > 1700000000); // After 2023
        assert!(TestTimestamps::YEAR_2024_END > TestTimestamps::YEAR_2024_START);
        assert!(TestTimestamps::FAR_FUTURE > TestTimestamps::YEAR_2024_END);
    }

    #[test]
    fn test_dynamic_timestamps() {
        let now = TestTimestamps::now();
        let future = TestTimestamps::thirty_days_from_now();
        let past = TestTimestamps::one_hour_ago();
        
        assert!(future > now);
        assert!(past < now);
    }

    #[test]
    fn test_permission_builders() {
        let analytics = PermissionBuilder::analytics_with_expiry(TestTimestamps::YEAR_2024_END);
        assert_eq!(analytics, "epsx:analytics:view:1735689599");
        
        let rankings = PermissionBuilder::rankings_with_limit_and_expiry(50, TestTimestamps::MID_2024);
        assert_eq!(rankings, "epsx:rankings:view:50:1717200000");
    }
}