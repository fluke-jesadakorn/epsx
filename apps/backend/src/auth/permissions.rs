use chrono::{DateTime, Utc};// ============================================================================
use uuid::Uuid;
use crate::core::constants::*;
// UNIFIED PERMISSION SYSTEM - REPLACES ALL ROLE-BASED ACCESS CONTROL
// ============================================================================
// This module implements a single permission-based access control system
// Format: "platform:resource:action" (e.g., "epsx:analytics:view")
// Platforms: epsx, epsx-pay, epsx-token, admin
// No role concept - only permission-based access control

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::shared_kernel::value_objects::{ResolvedUserLimits, UserDynamicLimit};



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
    
    
    pub fn matches(&self, other: &Permission) -> bool {
        // Support wildcards in permission matching
        (self.platform == "*" || other.platform == "*" || self.platform == other.platform) &&
        (self.resource == "*" || other.resource == "*" || self.resource == other.resource) &&
        (self.action == "*" || other.action == "*" || self.action == other.action)
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.platform, self.resource, self.action)
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
/// SECURITY: All timestamp validation is server-side only using UTC time
pub fn is_permission_valid_with_time_check(permission: &str) -> bool {
    let (_, timestamp) = parse_permission_with_timestamp(permission);
    
    match timestamp {
        Some(exp_time) => {
            let now = Utc::now().timestamp();
            // SECURITY: Additional validation - ensure timestamp is reasonable
            // Reject timestamps that are too far in the future (> 10 years)
            let max_future = now + (10 * YEAR); // 10 years
            if exp_time > max_future {
                tracing::warn!("Permission timestamp too far in future: {} > {}", exp_time, max_future);
                return false;
            }
            
            // Check if not expired
            now <= exp_time
        },
        None => true, // No timestamp = permanent permission
    }
}

// ============================================================================
// PERMISSION ACCESS LOGIC - WITH EMBEDDED TIMESTAMP SUPPORT
// ============================================================================

/// Optimized permission checking with caching and early termination
pub fn check_permission_access(user_permissions: &[String], required_permission: &str) -> bool {
    // Cache parsed required permission to avoid repeated parsing
    thread_local! {
        static PERMISSION_CACHE: std::cell::RefCell<std::collections::HashMap<String, Option<Permission>>> = 
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    
    // Parse required permission with caching
    let (required_base, _) = parse_permission_with_timestamp(required_permission);
    let required = PERMISSION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.entry(required_base.clone()).or_insert_with(|| {
            Permission::from_string(&required_base).ok()
        }).clone()
    });
    
    let required = match required {
        Some(perm) => perm,
        None => return false,
    };
    
    // Fast path: Check for exact matches first (most common case)
    if user_permissions.contains(&required_permission.to_string()) {
        return is_permission_valid_with_time_check(required_permission);
    }
    
    // Fast path: Check for wildcard admin permissions early
    for perm_str in user_permissions {
        if perm_str == "admin:*:*" || perm_str.starts_with("admin:*:") {
            return is_permission_valid_with_time_check(perm_str);
        }
    }
    
    // Standard path: Validate each permission with optimized parsing
    let mut valid_permissions = Vec::with_capacity(user_permissions.len());
    
    // Pre-filter valid permissions to avoid repeated timestamp checks
    for perm_str in user_permissions {
        if is_permission_valid_with_time_check(perm_str) {
            valid_permissions.push(perm_str);
        }
    }
    
    // Check permissions with optimized parsing
    for perm_str in valid_permissions {
        let (base_permission, _) = parse_permission_with_timestamp(perm_str);
        
        // Use cached parsing for base permissions
        let user_perm = PERMISSION_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.entry(base_permission.clone()).or_insert_with(|| {
                Permission::from_string(&base_permission).ok()
            }).clone()
        });
        
        if let Some(user_perm) = user_perm {
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

// DISABLED: Dynamic limits functionality - to be implemented
// (Now available through domain layer imports)

/// Default limits for different permission levels (fallback when no dynamic limits)
pub const DEFAULT_FREE_RANKING_LIMIT: i32 = 3;
pub const DEFAULT_FREE_API_MINUTE_LIMIT: i32 = 10;
pub const DEFAULT_FREE_API_HOUR_LIMIT: i32 = 100;

/// Resolve final user limits by combining dynamic assignments with permission-based defaults
pub fn resolve_user_limits(
    wallet_address: Uuid,
    user_permissions: &[String],
    dynamic_limits: Option<UserDynamicLimit>,
) -> ResolvedUserLimits {
    match dynamic_limits {
        Some(limits) => resolve_from_dynamic_assignment(wallet_address, limits),
        None => resolve_from_permissions(wallet_address, user_permissions),
    }
}

/// Create resolved limits from a dynamic database assignment
fn resolve_from_dynamic_assignment(
    wallet_address: Uuid,
    dynamic_limit: UserDynamicLimit,
) -> ResolvedUserLimits {
    ResolvedUserLimits {
        wallet_address: Some(wallet_address),
        ranking_limit: 10,
        api_minute_limit: dynamic_limit.limit_value,
        daily_limit: dynamic_limit.limit_value * 24,
        weekly_limit: dynamic_limit.limit_value * 24 * 7,
        monthly_limit: dynamic_limit.limit_value * 24 * 30,
        total_limit: dynamic_limit.limit_value * 24 * 365,
        has_premium_features: false,
        is_admin: false,
    }
}

/// Create resolved limits from user permissions (fallback when no dynamic limits)
fn resolve_from_permissions(
    wallet_address: Uuid,
    user_permissions: &[String],
) -> ResolvedUserLimits {
    let ranking_limit = extract_ranking_limit(user_permissions);
    let (_api_per_minute, api_per_hour) = derive_api_limits_from_ranking(ranking_limit);
    let _api_endpoints = derive_api_endpoints_from_permissions(user_permissions);

    ResolvedUserLimits {
        wallet_address: Some(wallet_address),
        ranking_limit,
        api_minute_limit: api_per_hour / 60,
        daily_limit: api_per_hour * HOURS_PER_DAY,
        weekly_limit: api_per_hour * HOURS_PER_DAY * DAYS_PER_WEEK,
        monthly_limit: api_per_hour * HOURS_PER_DAY * DAYS_PER_MONTH,
        total_limit: api_per_hour * HOURS_PER_DAY * DAYS_PER_YEAR,
        has_premium_features: user_permissions.iter().any(|p| p.contains("premium")),
        is_admin: user_permissions.iter().any(|p| p.starts_with("admin:")),
    }
}

/// Derive API rate limits from ranking limits
fn derive_api_limits_from_ranking(ranking_limit: i32) -> (i32, i32) {
    match ranking_limit {
        -1 => (-1, -1),                    // Unlimited
        0..=3 => (10, 100),               // Basic access
        4..=25 => (30, 1500),             // Standard access  
        26..=50 => (60, 5000),            // Premium access
        51..=100 => (120, 15000),         // Professional access
        _ => (1000, 50000),               // Enterprise access (101+)
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
    
    
    endpoints
}

/// Check if user has enhanced access beyond basic level
pub fn has_enhanced_access(user_permissions: &[String]) -> bool {
    extract_ranking_limit(user_permissions) > DEFAULT_FREE_RANKING_LIMIT
}

/// Check if user has unlimited access (helper for unlimited checks)  
pub fn has_unlimited_access(user_permissions: &[String]) -> bool {
    extract_ranking_limit(user_permissions) == -1
}

/// Get user's effective ranking limit (handles both dynamic and permission-based)
pub fn get_effective_ranking_limit(
    user_permissions: &[String],
    dynamic_limit: Option<&UserDynamicLimit>,
) -> i32 {
    match dynamic_limit {
        Some(limit) => {
            // Use limit_value if it's a ranking limit type
            if limit.limit_type == "ranking" {
                limit.limit_value
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
    dynamic_limit: Option<&UserDynamicLimit>,
) -> (i32, i32) {
    match dynamic_limit {
        Some(limit) => {
            // Use limit_value for API limits based on limit_type
            match limit.limit_type.as_str() {
                "api_minute" => (limit.limit_value, DEFAULT_FREE_API_HOUR_LIMIT),
                "api_hour" => (DEFAULT_FREE_API_MINUTE_LIMIT, limit.limit_value),
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
    _dynamic_limit: Option<&UserDynamicLimit>,
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
    
    if let Some(prefix) = pattern.strip_suffix("/*") {
        return endpoint.starts_with(prefix);
    }
    
    endpoint == pattern
}

/// Create a default resolved limits for a user (used as final fallback)
pub fn create_default_limits(wallet_address: Uuid) -> ResolvedUserLimits {
    ResolvedUserLimits {
        wallet_address: Some(wallet_address),
        ranking_limit: DEFAULT_FREE_RANKING_LIMIT,
        api_minute_limit: DEFAULT_FREE_API_MINUTE_LIMIT,
        daily_limit: DEFAULT_FREE_API_MINUTE_LIMIT * MINUTES_PER_DAY, // 24 hours worth of minute limits
        weekly_limit: DEFAULT_FREE_API_MINUTE_LIMIT * 1440 * 7,
        monthly_limit: DEFAULT_FREE_API_MINUTE_LIMIT * 1440 * 30,
        total_limit: DEFAULT_FREE_API_MINUTE_LIMIT * 1440 * 365,
        has_premium_features: false,
        is_admin: false,
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
/// SECURITY: Server-side timestamp creation with validation
pub fn add_timestamp_to_permission(base_permission: &str, hours_from_now: i64) -> Result<String, String> {
    // SECURITY: Validate input parameters
    if hours_from_now <= 0 {
        return Err("Hours from now must be positive".to_string());
    }
    
    if hours_from_now > (10 * 365 * 24) as i64 { // Max 10 years (87600 hours)
        return Err("Permission duration cannot exceed 10 years".to_string());
    }
    
    if base_permission.is_empty() {
        return Err("Base permission cannot be empty".to_string());
    }
    
    // SECURITY: Server-side timestamp generation only
    let expires_at = Utc::now().timestamp() + (hours_from_now * HOUR);
    Ok(format!("{}:{}", base_permission, expires_at))
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
pub fn create_temporary_admin_permission(base_permission: &str, hours: i64) -> Result<String, String> {
    add_timestamp_to_permission(base_permission, hours)
}

/// Extend permission expiry by additional hours
/// Works on both permanent (adds timestamp) and timed permissions (extends existing)
pub fn extend_permission_expiry(permission: &str, additional_hours: i64) -> Result<String, String> {
    let (base_perm, current_timestamp) = parse_permission_with_timestamp(permission);
    
    let new_expiry = match current_timestamp {
        Some(current) => current + (additional_hours * HOUR),
        None => Utc::now().timestamp() + (additional_hours * HOUR),
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
        diff / HOUR // Convert seconds to hours
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
        let future_time = Utc::now().timestamp() + HOUR; // 1 hour from now
        let future_perm = format!("admin:users:modify:{}", future_time);
        assert!(is_permission_valid_with_time_check(&future_perm));
        
        // Test past timestamp (expired)
        let past_time = Utc::now().timestamp() - HOUR; // 1 hour ago
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
        let unlimited_perms = vec!["epsx:rankings:view:unlimited".to_string(), "epsx:*:*".to_string()];
        assert_eq!(extract_ranking_limit(&unlimited_perms), -1);
        
        // Test default fallback when no ranking permission found
        let no_ranking_perms = vec!["epsx:trading:basic".to_string()];
        assert_eq!(extract_ranking_limit(&no_ranking_perms), 5);
    }

    #[test]
    fn test_enhanced_access() {
        // Test enhanced access logic
        let basic_perms = vec!["epsx:rankings:view:3".to_string()];
        assert!(!has_enhanced_access(&basic_perms));
        
        let standard_perms = vec!["epsx:rankings:view:25".to_string()];
        assert!(has_enhanced_access(&standard_perms));
        
        let unlimited_perms = vec!["epsx:rankings:view:unlimited".to_string()];
        assert!(has_enhanced_access(&unlimited_perms));
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

}