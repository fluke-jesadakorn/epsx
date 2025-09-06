use chrono::{DateTime, Utc};// ============================================================================
use uuid::Uuid;
// UNIFIED PERMISSION SYSTEM - REPLACES ALL ROLE-BASED ACCESS CONTROL
// ============================================================================
// This module implements a single permission-based access control system
// Format: "platform:resource:action" (e.g., "epsx:analytics:view")
// Platforms: epsx, epsx-pay, epsx-token, admin
// No role concept - only permission-based access control

use serde::{Deserialize, Serialize};



// ============================================================================
// PERMISSION STRUCTURE
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub platform: String,
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(platform: &str, resource: &str, action: &str) -> Self {
        Self {
            platform: platform.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }
    
    pub fn from_string(permission_str: &str) -> Result<Self, PermissionError> {
        let parts: Vec<&str> = permission_str.split(':').collect();
        
        if parts.len() != 3 {
            return Err(PermissionError::InvalidFormat(
                format!("Permission must be in format 'platform:resource:action', got: {}", permission_str)
            ));
        }
        
        Ok(Permission::new(parts[0], parts[1], parts[2]))
    }
    
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.platform, self.resource, self.action)
    }
    
    pub fn matches(&self, other: &Permission) -> bool {
        // Support wildcards in permission matching
        (self.platform == "*" || other.platform == "*" || self.platform == other.platform) &&
        (self.resource == "*" || other.resource == "*" || self.resource == other.resource) &&
        (self.action == "*" || other.action == "*" || self.action == other.action)
    }
}

// ============================================================================
// USER CLAIMS WITH PERMISSIONS ONLY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub firebase_uid: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
}

// ============================================================================
// EMBEDDED TIMESTAMP PERMISSION LOGIC
// ============================================================================

/// Parse permission with optional embedded timestamp
/// Format: "platform:resource:action" or "platform:resource:action:unix_timestamp"
/// Returns: (base_permission, optional_timestamp)
pub fn parse_permission_with_timestamp(permission: &str) -> (String, Option<i64>) {
    let parts: Vec<&str> = permission.split(':').collect();
    
    if parts.len() >= 4 {
        // Check if last part is unix timestamp
        if let Ok(timestamp) = parts.last().unwrap().parse::<i64>() {
            // Valid timestamp - return base permission + timestamp
            let base_permission = parts[..parts.len()-1].join(":");
            return (base_permission, Some(timestamp));
        }
    }
    
    // No timestamp or invalid - return as-is
    (permission.to_string(), None)
}

/// Check if permission is valid (not expired if timestamp present)
/// If no timestamp -> always valid (permanent)
/// If timestamp present -> check expiry
pub fn is_permission_valid_with_time_check(permission: &str) -> bool {
    let (_, timestamp) = parse_permission_with_timestamp(permission);
    
    match timestamp {
        Some(exp_time) => Utc::now().timestamp() <= exp_time, // Check expiry
        None => true, // No timestamp = permanent permission
    }
}

// ============================================================================
// PERMISSION ACCESS LOGIC - WITH EMBEDDED TIMESTAMP SUPPORT
// ============================================================================

pub fn check_permission_access(user_permissions: &[String], required_permission: &str) -> bool {
    // Parse required permission (may also have timestamp)
    let (required_base, _) = parse_permission_with_timestamp(required_permission);
    let required = match Permission::from_string(&required_base) {
        Ok(perm) => perm,
        Err(_) => return false,
    };
    
    for perm_str in user_permissions {
        // First check if this permission is expired
        if !is_permission_valid_with_time_check(perm_str) {
            continue; // Skip expired permissions
        }
        
        // Get base permission without timestamp
        let (base_permission, _) = parse_permission_with_timestamp(perm_str);
        
        // Check permission match using base permissions
        if let Ok(user_perm) = Permission::from_string(&base_permission) {
            if user_perm.matches(&required) {
                return true;
            }
        }
    }
    
    false
}

pub fn check_multiple_permissions(user_permissions: &[String], required_permissions: &[String]) -> bool {
    required_permissions.iter()
        .all(|perm| check_permission_access(user_permissions, perm))
}

pub fn check_any_permission(user_permissions: &[String], required_permissions: &[String]) -> bool {
    required_permissions.iter()
        .any(|perm| check_permission_access(user_permissions, perm))
}

// ============================================================================
// PREDEFINED PERMISSION SETS (REPLACES ROLE-BASED FEATURES)
// ============================================================================

pub struct PermissionSets;

impl PermissionSets {
    // Admin permissions - full access to everything
    pub fn admin() -> Vec<String> {
        vec!["admin:*:*".to_string()]
    }
    
    // Premium user permissions (replaces "user" role)
    pub fn premium_user() -> Vec<String> {
        vec![
            "epsx:analytics:view".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:realtime:access".to_string(),
            "epsx:profile:manage".to_string(),
            "epsx:notifications:receive".to_string(),
            "epsx:billing:manage".to_string(),
            "epsx:filters:advanced".to_string(),
        ]
    }
    
    // Basic user permissions (replaces "guest" role)
    pub fn basic_user() -> Vec<String> {
        vec!["epsx:analytics:view".to_string()]
    }
    
    // Platform-specific permissions
    pub fn pay_platform_access() -> Vec<String> {
        vec![
            "epsx-pay:payments:view".to_string(),
            "epsx-pay:payments:process".to_string(),
            "epsx-pay:transactions:view".to_string(),
        ]
    }
    
    pub fn token_platform_access() -> Vec<String> {
        vec![
            "epsx-token:tokens:view".to_string(),
            "epsx-token:governance:vote".to_string(),
            "epsx-token:tokens:stake".to_string(),
        ]
    }
    
    // ============================================================================
    // PACKAGE TIER REPLACEMENT PERMISSION SETS
    // ============================================================================
    
    // Bronze tier permissions (replaces BRONZE package)
    pub fn bronze_user() -> Vec<String> {
        vec![
            "epsx:rankings:view:5".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:notifications:basic".to_string(),
        ]
    }
    
    // Silver tier permissions (replaces SILVER package)
    pub fn silver_user() -> Vec<String> {
        vec![
            "epsx:rankings:view:25".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:history".to_string(),
            "epsx:notifications:enhanced".to_string(),
            "epsx:analytics:basic".to_string(),
            "epsx:alerts:email".to_string(),
        ]
    }
    
    // Gold tier permissions (replaces GOLD package)
    pub fn gold_user() -> Vec<String> {
        vec![
            "epsx:rankings:view:50".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:history".to_string(),
            "epsx:portfolio:tools".to_string(),
            "epsx:notifications:enhanced".to_string(),
            "epsx:analytics:basic".to_string(),
            "epsx:analytics:advanced".to_string(),
            "epsx:analytics:premium".to_string(),
            "epsx:alerts:email".to_string(),
            "epsx:support:priority".to_string(),
        ]
    }
    
    // Platinum tier permissions (replaces PLATINUM package)
    pub fn platinum_user() -> Vec<String> {
        vec![
            "epsx:rankings:view:100".to_string(),
            "epsx:trading:basic".to_string(),
            "epsx:trading:advanced".to_string(),
            "epsx:trading:premium".to_string(),
            "epsx:portfolio:view".to_string(),
            "epsx:portfolio:history".to_string(),
            "epsx:portfolio:tools".to_string(),
            "epsx:notifications:enhanced".to_string(),
            "epsx:analytics:basic".to_string(),
            "epsx:analytics:advanced".to_string(),
            "epsx:analytics:premium".to_string(),
            "epsx:alerts:email".to_string(),
            "epsx:support:priority".to_string(),
            "epsx:research:reports".to_string(),
            "epsx:dashboards:custom".to_string(),
        ]
    }
    
    // Enterprise tier permissions (replaces ENTERPRISE package)
    pub fn enterprise_user() -> Vec<String> {
        vec![
            "epsx:rankings:view:unlimited".to_string(),
            "epsx:*:*".to_string(),
            "epsx-pay:*:*".to_string(),
            "epsx-token:*:*".to_string(),
        ]
    }
}

// ============================================================================
// DYNAMIC RANKING LIMIT EXTRACTION
// ============================================================================

/// Extract ranking limit from permissions array
/// Parses permissions like "epsx:rankings:view:25" to return the numeric limit
pub fn extract_ranking_limit(user_permissions: &[String]) -> i32 {
    for perm in user_permissions {
        if let Some(limit_str) = perm.strip_prefix("epsx:rankings:view:") {
            if limit_str == "unlimited" {
                return -1; // -1 represents unlimited access
            }
            if let Ok(limit) = limit_str.parse::<i32>() {
                return limit;
            }
        }
    }
    5 // Default fallback to basic access (Bronze level)
}

/// Convert package tier string to permission set (for migration compatibility)
pub fn convert_package_tier_to_permissions(tier: &str) -> Vec<String> {
    match tier.to_uppercase().as_str() {
        "BRONZE" => PermissionSets::bronze_user(),
        "SILVER" => PermissionSets::silver_user(),
        "GOLD" => PermissionSets::gold_user(),
        "PLATINUM" => PermissionSets::platinum_user(),
        "ENTERPRISE" => PermissionSets::enterprise_user(),
        "VIP" => PermissionSets::enterprise_user(), // VIP maps to enterprise
        _ => PermissionSets::bronze_user(), // Default fallback
    }
}

/// Derive tier display name from ranking limit (for UI compatibility)
pub fn derive_tier_from_ranking_limit(limit: i32) -> String {
    match limit {
        5 => "BRONZE".to_string(),
        25 => "SILVER".to_string(),
        50 => "GOLD".to_string(),
        100 => "PLATINUM".to_string(),
        -1 => "ENTERPRISE".to_string(), // Unlimited
        _ => {
            // Handle custom limits
            if limit > 0 && limit <= 10 {
                "BRONZE".to_string()
            } else if limit <= 30 {
                "SILVER".to_string()
            } else if limit <= 75 {
                "GOLD".to_string()
            } else if limit <= 150 {
                "PLATINUM".to_string()
            } else {
                "ENTERPRISE".to_string()
            }
        }
    }
}

/// Check if user can view specific ranking position
pub fn can_view_ranking_position(user_permissions: &[String], position: i32) -> bool {
    let limit = extract_ranking_limit(user_permissions);
    
    // Unlimited access
    if limit == -1 {
        return true;
    }
    
    // Check if position is within limit
    position <= limit
}

// ============================================================================
// CLEAN ARCHITECTURE INTEGRATION - DATABASE MOVED TO INFRASTRUCTURE LAYER
// ============================================================================
// Note: Database integration functions moved to infrastructure layer
// This module now contains only pure domain logic for permission validation

// ============================================================================
// PERMISSION VALIDATION HELPERS
// ============================================================================

pub fn has_admin_access(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "admin:*:*")
}

pub fn can_view_analytics(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:analytics:view") ||
    has_admin_access(user_permissions)
}

pub fn can_export_data(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:analytics:export") ||
    has_admin_access(user_permissions)
}

pub fn can_access_realtime(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:realtime:access") ||
    has_admin_access(user_permissions)
}

pub fn can_manage_profile(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:profile:manage") ||
    has_admin_access(user_permissions)
}

pub fn can_receive_notifications(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:notifications:receive") ||
    has_admin_access(user_permissions)
}

pub fn can_manage_billing(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:billing:manage") ||
    has_admin_access(user_permissions)
}

pub fn can_use_advanced_filters(user_permissions: &[String]) -> bool {
    check_permission_access(user_permissions, "epsx:filters:advanced") ||
    has_admin_access(user_permissions)
}

// Platform access checks
pub fn can_access_pay_platform(user_permissions: &[String]) -> bool {
    check_any_permission(user_permissions, &[
        "epsx-pay:*:*".to_string(),
        "admin:*:*".to_string(),
    ])
}

pub fn can_access_token_platform(user_permissions: &[String]) -> bool {
    check_any_permission(user_permissions, &[
        "epsx-token:*:*".to_string(),
        "admin:*:*".to_string(),
    ])
}

// ============================================================================
// ERROR TYPES - PURE DOMAIN ERRORS
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Access denied: insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Permission not available")]
    PermissionNotAvailable,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid permission format: {0}")]
    InvalidFormat(String),
}

// ============================================================================
// MIDDLEWARE HELPERS - NOW PURE FUNCTIONS
// ============================================================================
// Note: Database-dependent middleware moved to infrastructure layer
// These are now pure functions for permission validation

pub fn require_permission_pure(
    user_permissions: &[String],
    required_permission: &str
) -> Result<(), PermissionError> {
    if check_permission_access(user_permissions, required_permission) {
        Ok(())
    } else {
        Err(PermissionError::InsufficientPermissions)
    }
}

// ============================================================================
// DYNAMIC LIMIT RESOLUTION - UNIFIED SYSTEM FOR ADMIN-CONTROLLED LIMITS
// ============================================================================
// This section implements the core logic for resolving user limits dynamically
// combining database-stored admin assignments with permission-based fallbacks

use crate::infrastructure::adapters::repositories::diesel::models::{

    DieselUserDynamicLimit, ResolvedUserLimits, LimitSource
};

/// Default limits for different permission levels (fallback when no dynamic limits)
pub const DEFAULT_FREE_RANKING_LIMIT: i32 = 3;
pub const DEFAULT_FREE_API_MINUTE_LIMIT: i32 = 10;
pub const DEFAULT_FREE_API_HOUR_LIMIT: i32 = 100;

/// Resolve final user limits by combining dynamic assignments with permission-based defaults
pub fn resolve_user_limits(
    user_id: Uuid,
    user_permissions: &[String],
    dynamic_limits: Option<DieselUserDynamicLimit>,
) -> ResolvedUserLimits {
    match dynamic_limits {
        Some(limits) => resolve_from_dynamic_assignment(user_id, limits),
        None => resolve_from_permissions(user_id, user_permissions),
    }
}

/// Create resolved limits from a dynamic database assignment
fn resolve_from_dynamic_assignment(
    user_id: Uuid,
    dynamic_limit: DieselUserDynamicLimit,
) -> ResolvedUserLimits {
    ResolvedUserLimits {
        user_id: user_id.to_string(),
        daily_limit: Some(dynamic_limit.limit_value),
        weekly_limit: None,
        monthly_limit: None,
        total_limit: None,
    }
}

/// Create resolved limits from user permissions (fallback when no dynamic limits)
fn resolve_from_permissions(
    user_id: Uuid,
    user_permissions: &[String],
) -> ResolvedUserLimits {
    let ranking_limit = extract_ranking_limit(user_permissions);
    let (api_per_minute, api_per_hour) = derive_api_limits_from_ranking(ranking_limit);
    let api_endpoints = derive_api_endpoints_from_permissions(user_permissions);

    ResolvedUserLimits {
        user_id: user_id.to_string(),
        daily_limit: Some(api_per_hour as i64),
        weekly_limit: None,
        monthly_limit: None,
        total_limit: None,
    }
}

/// Derive API rate limits from ranking limits (maintains tier relationship)
fn derive_api_limits_from_ranking(ranking_limit: i32) -> (i32, i32) {
    match ranking_limit {
        -1 => (-1, -1),                    // Unlimited
        0..=3 => (10, 100),               // Free tier
        4..=10 => (30, 500),              // Bronze tier  
        11..=30 => (60, 1500),            // Silver tier
        31..=75 => (120, 5000),           // Gold tier
        76..=150 => (300, 15000),         // Platinum tier
        _ => (1000, 50000),               // Enterprise tier (151+)
    }
}

/// Extract API endpoint patterns from user permissions
fn derive_api_endpoints_from_permissions(user_permissions: &[String]) -> Vec<String> {
    let mut endpoints = vec!["basic/*".to_string()]; // Default access

    // Check for specific permission-based endpoint access
    if check_permission_access(user_permissions, "epsx:trading:advanced") {
        endpoints.push("trading/advanced/*".to_string());
    }
    
    if check_permission_access(user_permissions, "epsx:trading:premium") {
        endpoints.push("trading/premium/*".to_string());
    }
    
    if check_permission_access(user_permissions, "epsx:analytics:premium") {
        endpoints.push("analytics/premium/*".to_string());
    }
    
    if check_any_permission(user_permissions, &[
        "epsx:enterprise:*".to_string(),
        "admin:*:*".to_string()
    ]) {
        endpoints.push("enterprise/*".to_string());
        endpoints.push("institutional/*".to_string());
    }
    
    endpoints
}

/// Check if user is above free tier (helper for simple tier comparisons)
pub fn is_above_free_tier(user_permissions: &[String]) -> bool {
    extract_ranking_limit(user_permissions) > DEFAULT_FREE_RANKING_LIMIT
}

/// Check if user has unlimited access (helper for unlimited checks)  
pub fn has_unlimited_access(user_permissions: &[String]) -> bool {
    extract_ranking_limit(user_permissions) == -1
}

/// Get user's effective ranking limit (handles both dynamic and permission-based)
pub fn get_effective_ranking_limit(
    user_permissions: &[String],
    dynamic_limit: Option<&DieselUserDynamicLimit>,
) -> i32 {
    match dynamic_limit {
        Some(limit) => {
            // Use limit_value if it's a ranking limit type
            if limit.limit_type == "ranking" {
                limit.limit_value as i32
            } else {
                DEFAULT_FREE_RANKING_LIMIT
            }
        },
        None => extract_ranking_limit(user_permissions),
    }
}

/// Get user's effective API rate limits (handles both dynamic and permission-based)
pub fn get_effective_api_limits(
    user_permissions: &[String],
    dynamic_limit: Option<&DieselUserDynamicLimit>,
) -> (i32, i32) {
    match dynamic_limit {
        Some(limit) => {
            // Use limit_value for API limits based on limit_type
            match limit.limit_type.as_str() {
                "api_minute" => (limit.limit_value as i32, DEFAULT_FREE_API_HOUR_LIMIT),
                "api_hour" => (DEFAULT_FREE_API_MINUTE_LIMIT, limit.limit_value as i32),
                _ => (DEFAULT_FREE_API_MINUTE_LIMIT, DEFAULT_FREE_API_HOUR_LIMIT),
            }
        },
        None => {
            let ranking_limit = extract_ranking_limit(user_permissions);
            derive_api_limits_from_ranking(ranking_limit)
        }
    }
}

/// Validate if user can access a specific endpoint (dynamic-aware)
pub fn can_access_endpoint(
    user_permissions: &[String], 
    dynamic_limit: Option<&DieselUserDynamicLimit>,
    endpoint: &str
) -> bool {
    // Dynamic limits don't contain endpoint restrictions in the current schema
    // Fall back to permission-based endpoint access
    let allowed_endpoints = derive_api_endpoints_from_permissions(user_permissions);
    allowed_endpoints.iter().any(|pattern| endpoint_matches_pattern(endpoint, pattern))
}

/// Check if an endpoint matches a pattern (supports wildcards)
fn endpoint_matches_pattern(endpoint: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "**" {
        return true;
    }
    
    if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        return endpoint.starts_with(prefix);
    }
    
    endpoint == pattern
}

/// Create a default resolved limits for a user (used as final fallback)
pub fn create_default_limits(user_id: Uuid) -> ResolvedUserLimits {
    ResolvedUserLimits {
        user_id: user_id.to_string(),
        daily_limit: Some(DEFAULT_FREE_API_MINUTE_LIMIT as i64 * 1440), // 24 hours worth of minute limits
        weekly_limit: None,
        monthly_limit: None,
        total_limit: None,
    }
}

pub fn require_any_permission_pure(
    user_permissions: &[String],
    required_permissions: &[String]
) -> Result<(), PermissionError> {
    if check_any_permission(user_permissions, required_permissions) {
        Ok(())
    } else {
        Err(PermissionError::InsufficientPermissions)
    }
}

// ============================================================================
// TIMESTAMP PERMISSION MANAGEMENT HELPERS
// ============================================================================

/// Add timestamp to permission (creates timed permission)
/// Example: add_timestamp_to_permission("admin:users:modify", 24) -> "admin:users:modify:1703980800"
pub fn add_timestamp_to_permission(base_permission: &str, hours_from_now: i64) -> String {
    let expires_at = Utc::now().timestamp() + (hours_from_now * 3600);
    format!("{}:{}", base_permission, expires_at)
}

/// Remove timestamp from permission (convert timed to permanent)
/// Example: remove_timestamp_from_permission("admin:users:modify:1703980800") -> "admin:users:modify"
pub fn remove_timestamp_from_permission(permission: &str) -> String {
    let (base_permission, _) = parse_permission_with_timestamp(permission);
    base_permission
}

/// Get expiry time from permission (if has timestamp)
/// Returns None if permanent permission
pub fn get_permission_expiry_time(permission: &str) -> Option<DateTime<Utc>> {
    let (_, timestamp) = parse_permission_with_timestamp(permission);
    timestamp.and_then(|ts| DateTime::from_timestamp(ts, 0))
}

/// Create temporary admin permission with specified duration
/// Example: create_temporary_admin_permission("admin:users:modify", 24) -> "admin:users:modify:1703980800"
pub fn create_temporary_admin_permission(base_permission: &str, hours: i64) -> String {
    add_timestamp_to_permission(base_permission, hours)
}

/// Extend permission expiry by additional hours
/// Works on both permanent (adds timestamp) and timed permissions (extends existing)
pub fn extend_permission_expiry(permission: &str, additional_hours: i64) -> Result<String, String> {
    let (base_perm, current_timestamp) = parse_permission_with_timestamp(permission);
    
    let new_expiry = match current_timestamp {
        Some(current) => current + (additional_hours * 3600),
        None => Utc::now().timestamp() + (additional_hours * 3600),
    };
    
    Ok(format!("{}:{}", base_perm, new_expiry))
}

/// Check how many hours until permission expires
/// Returns None for permanent permissions, Some(hours) for timed permissions
pub fn hours_until_expiry(permission: &str) -> Option<i64> {
    let (_, timestamp) = parse_permission_with_timestamp(permission);
    timestamp.map(|exp_time| {
        let now = Utc::now().timestamp();
        let diff = exp_time - now;
        diff / 3600 // Convert seconds to hours
    })
}

/// Filter out expired permissions from a permission list
pub fn filter_valid_permissions(permissions: &[String]) -> Vec<String> {
    permissions.iter()
        .filter(|perm| is_permission_valid_with_time_check(perm))
        .cloned()
        .collect()
}

/// Get all permissions that will expire within specified hours
pub fn get_expiring_permissions(permissions: &[String], within_hours: i64) -> Vec<String> {
    permissions.iter()
        .filter(|perm| {
            if let Some(hours_left) = hours_until_expiry(perm) {
                hours_left <= within_hours && hours_left > 0
            } else {
                false // Permanent permissions don't expire
            }
        })
        .cloned()
        .collect()
}



// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::from_string("epsx:analytics:view").unwrap();
        assert_eq!(perm.platform, "epsx");
        assert_eq!(perm.resource, "analytics");
        assert_eq!(perm.action, "view");
        assert_eq!(perm.to_string(), "epsx:analytics:view");
    }

    #[test]
    fn test_permission_matching() {
        let admin_perm = Permission::from_string("admin:*:*").unwrap();
        let specific_perm = Permission::from_string("epsx:analytics:view").unwrap();
        
        assert!(admin_perm.matches(&specific_perm));
        assert!(!specific_perm.matches(&admin_perm));
    }

    #[test]
    fn test_permission_access() {
        let user_permissions = vec!["epsx:analytics:view".to_string(), "epsx:analytics:export".to_string()];
        
        assert!(check_permission_access(&user_permissions, "epsx:analytics:view"));
        assert!(check_permission_access(&user_permissions, "epsx:analytics:export"));
        assert!(!check_permission_access(&user_permissions, "epsx:billing:manage"));
    }

    #[test]
    fn test_admin_access() {
        let admin_permissions = vec!["admin:*:*".to_string()];
        
        assert!(check_permission_access(&admin_permissions, "epsx:analytics:view"));
        assert!(check_permission_access(&admin_permissions, "epsx-pay:payments:process"));
        assert!(check_permission_access(&admin_permissions, "admin:users:manage"));
    }

    #[test]
    fn test_embedded_timestamp_parsing() {
        // Test permanent permission (no timestamp)
        let (base, timestamp) = parse_permission_with_timestamp("admin:users:modify");
        assert_eq!(base, "admin:users:modify");
        assert_eq!(timestamp, None);
        
        // Test timed permission (with timestamp)
        let (base, timestamp) = parse_permission_with_timestamp("admin:users:modify:1703980800");
        assert_eq!(base, "admin:users:modify");
        assert_eq!(timestamp, Some(1703980800));
        
        // Test complex permission with timestamp
        let (base, timestamp) = parse_permission_with_timestamp("epsx:rankings:view:25:1703980800");
        assert_eq!(base, "epsx:rankings:view:25");
        assert_eq!(timestamp, Some(1703980800));
        
        // Test invalid timestamp (not a number)
        let (base, timestamp) = parse_permission_with_timestamp("admin:users:modify:invalid");
        assert_eq!(base, "admin:users:modify:invalid");
        assert_eq!(timestamp, None);
    }
    
    #[test]
    fn test_permission_expiry_validation() {
        // Test permanent permission (always valid)
        assert!(is_permission_valid_with_time_check("admin:users:modify"));
        
        // Test future timestamp (valid)
        let future_time = Utc::now().timestamp() + 3600; // 1 hour from now
        let future_perm = format!("admin:users:modify:{}", future_time);
        assert!(is_permission_valid_with_time_check(&future_perm));
        
        // Test past timestamp (expired)
        let past_time = Utc::now().timestamp() - 3600; // 1 hour ago
        let expired_perm = format!("admin:users:modify:{}", past_time);
        assert!(!is_permission_valid_with_time_check(&expired_perm));
    }
    
    #[test]
    fn test_timestamp_permission_checking() {
        let future_time = Utc::now().timestamp() + 3600;
        let past_time = Utc::now().timestamp() - 3600;
        
        let permissions = vec![
            "admin:users:view".to_string(),                           // permanent
            format!("admin:users:modify:{}", future_time),          // valid timed
            format!("admin:users:delete:{}", past_time),            // expired timed
        ];
        
        // Permanent permission should work
        assert!(check_permission_access(&permissions, "admin:users:view"));
        
        // Valid timed permission should work
        assert!(check_permission_access(&permissions, "admin:users:modify"));
        
        // Expired timed permission should not work
        assert!(!check_permission_access(&permissions, "admin:users:delete"));
        
        // Admin wildcard should work with valid permissions
        assert!(check_permission_access(&permissions, "admin:users:create"));
    }
    
    #[test]
    fn test_permission_management_helpers() {
        // Test adding timestamp
        let timed_perm = add_timestamp_to_permission("admin:users:modify", 24);
        assert!(timed_perm.starts_with("admin:users:modify:"));
        assert!(timed_perm.len() > "admin:users:modify:".len());
        
        // Test removing timestamp
        let base_perm = remove_timestamp_from_permission(&timed_perm);
        assert_eq!(base_perm, "admin:users:modify");
        
        // Test permanent permission removal (no-op)
        let permanent = remove_timestamp_from_permission("admin:users:modify");
        assert_eq!(permanent, "admin:users:modify");
        
        // Test expiry time extraction
        let future_time = Utc::now().timestamp() + 3600;
        let future_perm = format!("admin:users:modify:{}", future_time);
        let expiry = get_permission_expiry_time(&future_perm);
        assert!(expiry.is_some());
        
        // Test permanent permission expiry (none)
        let expiry = get_permission_expiry_time("admin:users:modify");
        assert!(expiry.is_none());
    }
    
    #[test]
    fn test_filter_valid_permissions() {
        let future_time = Utc::now().timestamp() + 3600;
        let past_time = Utc::now().timestamp() - 3600;
        
        let permissions = vec![
            "admin:users:view".to_string(),                          // permanent
            format!("admin:users:modify:{}", future_time),         // valid
            format!("admin:users:delete:{}", past_time),           // expired
            "epsx:analytics:view".to_string(),                      // permanent
        ];
        
        let valid_perms = filter_valid_permissions(&permissions);
        assert_eq!(valid_perms.len(), 3); // Should exclude the expired one
        assert!(valid_perms.contains(&"admin:users:view".to_string()));
        assert!(valid_perms.contains(&"epsx:analytics:view".to_string()));
        assert!(!valid_perms.iter().any(|p| p.contains(&past_time.to_string())));
    }

    #[test]
    fn test_ranking_limit_extraction() {
        // Test basic ranking limits
        let bronze_perms = vec!["epsx:rankings:view:5".to_string(), "epsx:trading:basic".to_string()];
        assert_eq!(extract_ranking_limit(&bronze_perms), 5);
        
        let gold_perms = vec!["epsx:rankings:view:50".to_string(), "epsx:trading:advanced".to_string()];
        assert_eq!(extract_ranking_limit(&gold_perms), 50);
        
        // Test unlimited access
        let enterprise_perms = vec!["epsx:rankings:view:unlimited".to_string(), "epsx:*:*".to_string()];
        assert_eq!(extract_ranking_limit(&enterprise_perms), -1);
        
        // Test default fallback when no ranking permission found
        let no_ranking_perms = vec!["epsx:trading:basic".to_string()];
        assert_eq!(extract_ranking_limit(&no_ranking_perms), 5);
    }

    #[test]
    fn test_tier_derivation() {
        assert_eq!(derive_tier_from_ranking_limit(5), "BRONZE");
        assert_eq!(derive_tier_from_ranking_limit(25), "SILVER");
        assert_eq!(derive_tier_from_ranking_limit(50), "GOLD");
        assert_eq!(derive_tier_from_ranking_limit(100), "PLATINUM");
        assert_eq!(derive_tier_from_ranking_limit(-1), "ENTERPRISE");
        
        // Test custom limits fall into appropriate tiers
        assert_eq!(derive_tier_from_ranking_limit(8), "BRONZE"); // Custom but in bronze range
        assert_eq!(derive_tier_from_ranking_limit(30), "SILVER"); // Custom but in silver range
        assert_eq!(derive_tier_from_ranking_limit(75), "GOLD"); // Custom but in gold range
    }

    #[test]
    fn test_ranking_position_access() {
        let bronze_perms = vec!["epsx:rankings:view:5".to_string()];
        assert!(can_view_ranking_position(&bronze_perms, 3)); // Within limit
        assert!(can_view_ranking_position(&bronze_perms, 5)); // At limit
        assert!(!can_view_ranking_position(&bronze_perms, 6)); // Beyond limit
        
        let unlimited_perms = vec!["epsx:rankings:view:unlimited".to_string()];
        assert!(can_view_ranking_position(&unlimited_perms, 1000)); // Should allow any position
    }

    #[test]
    fn test_package_tier_conversion() {
        let bronze_perms = convert_package_tier_to_permissions("BRONZE");
        assert!(bronze_perms.contains(&"epsx:rankings:view:5".to_string()));
        assert!(bronze_perms.contains(&"epsx:trading:basic".to_string()));
        
        let gold_perms = convert_package_tier_to_permissions("gold"); // Test case insensitive
        assert!(gold_perms.contains(&"epsx:rankings:view:50".to_string()));
        assert!(gold_perms.contains(&"epsx:trading:premium".to_string()));
        
        // Test fallback for unknown tier
        let unknown_perms = convert_package_tier_to_permissions("UNKNOWN");
        assert_eq!(unknown_perms, PermissionSets::bronze_user());
    }
}