// Enhanced permission middleware for API endpoint access control with permission profiles

use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde_json::json;

use crate::web::middleware::auth_middleware::AuthCtx;
use crate::web::middleware::rate_limiter::{RateLimitConfig, InMemoryRateLimiter};
use crate::web::auth::AppState;
use crate::dom::entities::permission_profile::PermissionProfile;
use crate::dom::values::{UserId, Role};

/// API endpoint configuration
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub path_pattern: String,
    pub required_permissions: Vec<String>,
    pub rate_limits: RateLimitConfig,
    pub allow_wildcards: bool,
}

/// Enhanced permission middleware that checks API endpoint access and applies rate limiting
pub async fn permission_middleware(
    State(app_state): State<AppState>,
    auth_ctx: Option<AuthCtx>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip permission check for public endpoints
    let path = req.uri().path();
    let method = req.method();
    
    if is_public_endpoint(path, method) {
        return Ok(next.run(req).await);
    }
    
    // Require authentication for protected endpoints
    let auth_ctx = auth_ctx.ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Check API endpoint permissions
    let has_access = check_api_endpoint_access(
        &app_state,
        &auth_ctx.user_id,
        path,
        method,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_access.allowed {
        tracing::warn!(
            "API access denied for user {} to {} {}: {}",
            auth_ctx.user_id,
            method,
            path,
            has_access.reason
        );
        
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Apply rate limiting based on user's permission profiles
    let rate_limit_result = apply_rate_limiting(
        &app_state,
        &auth_ctx.user_id,
        path,
        method,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !rate_limit_result.allowed {
        tracing::warn!(
            "Rate limit exceeded for user {} to {} {}: {}",
            auth_ctx.user_id,
            method,
            path,
            rate_limit_result.reason
        );
        
        let response = Json(json!({
            "error": "Rate limit exceeded",
            "message": rate_limit_result.reason,
            "retry_after": rate_limit_result.retry_after_seconds
        }));
        
        return Ok(response.into_response());
    }
    
    // Add user context to request extensions for downstream handlers
    req.extensions_mut().insert(auth_ctx);
    
    Ok(next.run(req).await)
}

/// Check if user has access to a specific API endpoint
async fn check_api_endpoint_access(
    app_state: &AppState,
    user_id: &UserId,
    path: &str,
    method: &Method,
) -> Result<AccessCheckResult, PermissionError> {
    // Get user's permission profiles
    let user_permission_profiles = get_user_permission_profiles(app_state, user_id).await?;
    
    // Check against each permission profile's API endpoint configuration
    for permission_profile in &user_permission_profiles {
        if let Some(api_config) = get_api_endpoint_config(permission_profile) {
            if endpoint_matches_pattern(path, method, &api_config.allowed_endpoints) {
                return Ok(AccessCheckResult {
                    allowed: true,
                    reason: format!("Access granted via permission profile: {}", permission_profile.name()),
                });
            }
        }
    }
    
    // Check role-based access as fallback
    let user = app_state.user_repo.find_by_id(user_id).await
        .map_err(|e| PermissionError::UserNotFound(format!("Failed to find user: {}", e)))?;
    
    if has_role_based_access(user.role(), path, method) {
        return Ok(AccessCheckResult {
            allowed: true,
            reason: "Access granted via role-based permissions".to_string(),
        });
    }
    
    Ok(AccessCheckResult {
        allowed: false,
        reason: format!("No permission profile or role grants access to {} {}", method, path),
    })
}

/// Apply rate limiting based on user's permission profiles
async fn apply_rate_limiting(
    app_state: &AppState,
    user_id: &UserId,
    path: &str,
    method: &Method,
) -> Result<RateLimitResultInternal, PermissionError> {
    // Get rate limits from user's permission profiles
    let rate_limits = get_user_rate_limits(app_state, user_id).await?;
    
    // Apply the most generous rate limit (highest values)
    let effective_limit = rate_limits.into_iter()
        .fold(RateLimitConfig::default(), |acc, limit| {
            RateLimitConfig {
                requests_per_minute: max_option(acc.requests_per_minute, limit.requests_per_minute),
                requests_per_hour: max_option(acc.requests_per_hour, limit.requests_per_hour),
                requests_per_day: max_option(acc.requests_per_day, limit.requests_per_day),
            }
        });
    
    // Use the in-memory rate limiter
    // In production, this would be injected as a dependency
    static RATE_LIMITER: once_cell::sync::Lazy<InMemoryRateLimiter> = 
        once_cell::sync::Lazy::new(|| InMemoryRateLimiter::new());
    
    let result = RATE_LIMITER.check_rate_limit(
        user_id,
        path,
        method.as_str(),
        &effective_limit,
    ).await.map_err(|e| PermissionError::RateLimitError(e.to_string()))?;
    
    Ok(RateLimitResultInternal {
        allowed: result.allowed,
        reason: result.reason,
        retry_after_seconds: result.retry_after_seconds,
    })
}

/// Get user's permission profiles with API endpoint configurations
async fn get_user_permission_profiles(
    app_state: &AppState,
    user_id: &UserId,
) -> Result<Vec<PermissionProfile>, PermissionError> {
    // In a real implementation, this would query the user_features table
    // to get active permission profiles for the user
    
    // For demonstration, we'll get default permission profiles based on user role
    let user = app_state.user_repo.find_by_id(user_id).await
        .map_err(|e| PermissionError::UserNotFound(format!("Failed to find user: {}", e)))?;
    
    let default_permission_profiles = get_default_permission_profiles_for_role(user.role());
    
    // Filter to active permission profiles only
    let active_permission_profiles = default_permission_profiles.into_iter()
        .filter(|t| t.is_active())
        .collect();
    
    Ok(active_permission_profiles)
}

/// Get API endpoint configuration from permission profile metadata
fn get_api_endpoint_config(permission_profile: &PermissionProfile) -> Option<ApiEndpointConfig> {
    // In the real implementation, this would parse the api_endpoints JSONB field
    // from the permission profile metadata
    
    // For demonstration, return default configurations based on permission profile name
    match permission_profile.name() {
        "Bronze User" => Some(ApiEndpointConfig {
            allowed_endpoints: vec![
                EndpointPattern::new("/api/v1/auth/*", vec!["GET", "POST"]),
                EndpointPattern::new("/api/v1/market-data/basic", vec!["GET"]),
                EndpointPattern::new("/api/v1/analytics/eps", vec!["GET"]),
                // Stock ranking endpoints for Bronze users (5 rankings max)
                EndpointPattern::new("/api/v1/rankings/basic", vec!["GET"]),
                EndpointPattern::new("/api/v1/rankings/eps-growth", vec!["GET"]),
                EndpointPattern::new("/api/v1/rankings/market-cap", vec!["GET"]),
                EndpointPattern::new("/api/v1/rankings/volume", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/bronze", vec!["GET"]),
            ],
            rate_limits: RateLimitConfig {
                requests_per_minute: Some(10), // Bronze: 10 requests/min for stock rankings
                requests_per_hour: Some(100),
                requests_per_day: Some(1000),
            },
        }),
        "Silver User" => Some(ApiEndpointConfig {
            allowed_endpoints: vec![
                EndpointPattern::new("/api/v1/auth/*", vec!["GET", "POST"]),
                EndpointPattern::new("/api/v1/market-data/*", vec!["GET"]),
                EndpointPattern::new("/api/v1/analytics/*", vec!["GET"]),
                EndpointPattern::new("/api/v1/alerts/basic", vec!["GET", "POST"]),
                // Stock ranking endpoints for Silver users (25 rankings max)
                EndpointPattern::new("/api/v1/rankings/*", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/silver", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/technical-indicators", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/price-change", vec!["GET"]),
                EndpointPattern::new("/api/v1/market-data/stocks/screener", vec!["GET"]),
            ],
            rate_limits: RateLimitConfig {
                requests_per_minute: Some(50), // Silver: 50 requests/min for stock rankings
                requests_per_hour: Some(500),
                requests_per_day: Some(5000),
            },
        }),
        "Gold User" => Some(ApiEndpointConfig {
            allowed_endpoints: vec![
                EndpointPattern::new("/api/v1/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/admin/analytics/*", vec!["GET"]),
                // Premium stock ranking endpoints for Gold users (50 rankings max)
                EndpointPattern::new("/api/v1/premium/rankings/*", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/gold", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/ai-insights", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/pattern-recognition", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/custom-metrics", vec!["GET"]),
                EndpointPattern::new("/api/v1/stock-rankings/advanced", vec!["GET"]),
            ],
            rate_limits: RateLimitConfig {
                requests_per_minute: Some(200), // Gold: 200 requests/min for stock rankings
                requests_per_hour: Some(2000),
                requests_per_day: Some(20000),
            },
        }),
        "Platinum User" => Some(ApiEndpointConfig {
            allowed_endpoints: vec![
                EndpointPattern::new("/api/v1/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/admin/*", vec!["GET"]),
                // All stock ranking endpoints for Platinum users (100 rankings max)
                EndpointPattern::new("/api/v1/premium/rankings/*", vec!["GET", "POST"]),
                EndpointPattern::new("/api/v1/stock-rankings/*", vec!["GET", "POST"]),
                EndpointPattern::new("/api/v1/custom-rankings/*", vec!["GET", "POST", "PUT", "DELETE"]),
            ],
            rate_limits: RateLimitConfig {
                requests_per_minute: Some(500), // Platinum: 500 requests/min for stock rankings
                requests_per_hour: Some(5000),
                requests_per_day: Some(50000),
            },
        }),
        "Enterprise User" => Some(ApiEndpointConfig {
            allowed_endpoints: vec![
                EndpointPattern::new("/api/v1/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/admin/*", vec!["GET", "POST"]),
                // Unlimited stock ranking access for Enterprise users
                EndpointPattern::new("/api/v1/premium/rankings/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/v1/stock-rankings/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/v1/custom-rankings/*", vec!["GET", "POST", "PUT", "DELETE"]),
                EndpointPattern::new("/api/v1/institutional-rankings/*", vec!["GET", "POST", "PUT", "DELETE"]),
            ],
            rate_limits: RateLimitConfig {
                requests_per_minute: Some(1000), // Enterprise: 1000 requests/min
                requests_per_hour: Some(10000),
                requests_per_day: Some(100000),
            },
        }),
        _ => None,
    }
}

/// Get rate limits for a user based on their permission profiles
async fn get_user_rate_limits(
    app_state: &AppState,
    user_id: &UserId,
) -> Result<Vec<RateLimitConfig>, PermissionError> {
    let permission_profiles = get_user_permission_profiles(app_state, user_id).await?;
    
    let rate_limits = permission_profiles.iter()
        .filter_map(|t| get_api_endpoint_config(t))
        .map(|config| config.rate_limits)
        .collect();
    
    Ok(rate_limits)
}

/// Check if endpoint matches any of the allowed patterns
fn endpoint_matches_pattern(
    path: &str,
    method: &Method,
    patterns: &[EndpointPattern],
) -> bool {
    for pattern in patterns {
        if pattern.matches(path, method) {
            return true;
        }
    }
    false
}

/// Check if user has role-based access (fallback for admin users)
fn has_role_based_access(role: &Role, path: &str, method: &Method) -> bool {
    match role {
        Role::SuperAdmin => true, // Super admin has access to everything
        Role::Admin => {
            // Admin has access to most endpoints except system-level operations
            !path.starts_with("/api/v1/system/") || method == &Method::GET
        }
        Role::User => {
            // Regular users only have basic access
            path.starts_with("/api/v1/auth/") || 
            path.starts_with("/api/v1/market-data/basic") ||
            (path.starts_with("/api/v1/analytics/") && method == &Method::GET)
        }
        Role::Free => {
            // Free users have very limited access
            path.starts_with("/api/v1/auth/") || 
            path == "/api/v1/market-data/basic"
        }
        Role::Premium => {
            // Premium users have enhanced access
            path.starts_with("/api/v1/auth/") || 
            path.starts_with("/api/v1/market-data/") ||
            path.starts_with("/api/v1/analytics/") ||
            path.starts_with("/api/v1/alerts/")
        }
        Role::Moderator => {
            // Moderators have user access plus moderation endpoints
            has_role_based_access(&Role::Premium, path, method) ||
            path.starts_with("/api/v1/moderation/")
        }
        Role::ApiClient => {
            // API clients have limited access to API endpoints only
            path.starts_with("/api/v1/") && !path.starts_with("/api/v1/admin/")
        }
    }
}

/// Check if endpoint is public (doesn't require authentication)
fn is_public_endpoint(path: &str, method: &Method) -> bool {
    let public_patterns = [
        ("/health", &Method::GET),
        ("/api/v1/auth/login", &Method::POST),
        ("/api/v1/auth/register", &Method::POST),
        ("/api/v1/auth/password-reset", &Method::POST),
        ("/login", &Method::POST),
        ("/register", &Method::POST),
        ("/password-reset", &Method::POST),
        ("/auth/me-public", &Method::GET),
    ];
    
    public_patterns.iter().any(|(pattern, allowed_method)| {
        path == *pattern && method == *allowed_method
    })
}

// Helper function to get default permission profiles for role (temporary implementation)
fn get_default_permission_profiles_for_role(role: &Role) -> Vec<PermissionProfile> {
    use crate::dom::entities::permission_profile::DefaultPermissionProfiles;
    use crate::dom::values::UserId;
    
    let creator_id = UserId::new("system".to_string());
    
    match role {
        Role::Free => vec![DefaultPermissionProfiles::bronze_user(creator_id)],
        Role::User => vec![DefaultPermissionProfiles::bronze_user(creator_id)],
        Role::Premium => vec![DefaultPermissionProfiles::silver_user(creator_id)],
        Role::Moderator => vec![
            DefaultPermissionProfiles::silver_user(creator_id.clone()),
            DefaultPermissionProfiles::content_moderator(creator_id),
        ],
        Role::Admin => vec![
            DefaultPermissionProfiles::gold_user(creator_id.clone()),
            DefaultPermissionProfiles::admin_assistant(creator_id),
        ],
        Role::SuperAdmin => vec![
            DefaultPermissionProfiles::gold_user(creator_id.clone()),
            DefaultPermissionProfiles::admin_assistant(creator_id),
        ],
        Role::ApiClient => vec![
            DefaultPermissionProfiles::bronze_user(creator_id), // API clients get basic access
        ],
    }
}

fn max_option(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

// Supporting types and structs

#[derive(Debug, Clone)]
struct ApiEndpointConfig {
    allowed_endpoints: Vec<EndpointPattern>,
    rate_limits: RateLimitConfig,
}

#[derive(Debug)]
struct RateLimitResultInternal {
    allowed: bool,
    reason: String,
    retry_after_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
struct EndpointPattern {
    path_pattern: String,
    allowed_methods: Vec<String>,
}

impl EndpointPattern {
    fn new(path_pattern: &str, methods: Vec<&str>) -> Self {
        Self {
            path_pattern: path_pattern.to_string(),
            allowed_methods: methods.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    fn matches(&self, path: &str, method: &Method) -> bool {
        // Check method
        if !self.allowed_methods.contains(&method.as_str().to_string()) {
            return false;
        }
        
        // Check path pattern with wildcard support
        if self.path_pattern.ends_with("/*") {
            let prefix = &self.path_pattern[..self.path_pattern.len() - 2];
            path.starts_with(prefix)
        } else if self.path_pattern.ends_with("*") {
            let prefix = &self.path_pattern[..self.path_pattern.len() - 1];
            path.starts_with(prefix)
        } else {
            path == self.path_pattern
        }
    }
}

#[derive(Debug)]
struct AccessCheckResult {
    allowed: bool,
    reason: String,
}


#[derive(Debug, thiserror::Error)]
enum PermissionError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_endpoint_pattern_matching() {
        let pattern = EndpointPattern::new("/api/v1/auth/*", vec!["GET", "POST"]);
        
        // Should match
        assert!(pattern.matches("/api/v1/auth/login", &Method::POST));
        assert!(pattern.matches("/api/v1/auth/profile", &Method::GET));
        
        // Should not match
        assert!(!pattern.matches("/api/v1/auth/login", &Method::DELETE));
        assert!(!pattern.matches("/api/v1/market-data/basic", &Method::GET));
    }
    
    #[test]
    fn test_public_endpoint_detection() {
        assert!(is_public_endpoint("/health", &Method::GET));
        assert!(is_public_endpoint("/api/v1/auth/login", &Method::POST));
        assert!(!is_public_endpoint("/api/v1/auth/login", &Method::GET));
        assert!(!is_public_endpoint("/api/v1/market-data", &Method::GET));
    }
    
    #[test]
    fn test_role_based_access() {
        assert!(has_role_based_access(&Role::SuperAdmin, "/api/v1/system/config", &Method::POST));
        assert!(has_role_based_access(&Role::Admin, "/api/v1/users", &Method::GET));
        assert!(!has_role_based_access(&Role::User, "/api/v1/admin/users", &Method::GET));
    }
}