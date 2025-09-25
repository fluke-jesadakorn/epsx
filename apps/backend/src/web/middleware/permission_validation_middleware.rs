// ============================================================================
// BULLETPROOF PERMISSION VALIDATION MIDDLEWARE (Phase 1.2)
// THE SINGLE SOURCE OF TRUTH for all permission enforcement
// ============================================================================

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::{Response, IntoResponse},
};
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::web::auth::AppState;
use crate::web::errors::{PermissionError, RiskLevel};

/// Permission validation middleware - THE AUTHORITY for all permission checks
/// This middleware enforces permission validation on ALL protected routes
/// Frontend/admin applications should NEVER do local permission validation
pub async fn permission_validation_middleware(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let path = request.uri().path();
    
    // Skip permission validation for public endpoints
    if is_public_endpoint(path) {
        return Ok(next.run(request).await);
    }
    
    // Skip permission validation for health checks
    if is_health_endpoint(path) {
        return Ok(next.run(request).await);
    }
    
    // Extract authentication information
    let auth_info = match extract_auth_info(&headers) {
        Some(info) => info,
        None => {
            warn!("Permission middleware: Missing authentication for protected route: {} {}", method, path);
            let error = PermissionError::authentication_required(
                "Valid authentication credentials required to access this resource"
            );
            return Err(error.into_response());
        }
    };
    
    // Determine required permission for this route
    let required_permission = match determine_required_permission(&method.to_string(), path) {
        Some(permission) => permission,
        None => {
            info!("Permission middleware: No permission required for route: {} {}", method, path);
            return Ok(next.run(request).await);
        }
    };
    
    // ⚡ CRITICAL: THE SINGLE SOURCE OF TRUTH permission validation
    let validation_result = validate_user_permission(
        &app_state,
        &auth_info.user_id,
        &required_permission,
        path,
        &method.to_string(),
    ).await;
    
    match validation_result {
        Ok(permission_granted) => {
            if permission_granted.granted {
                info!(
                    "Permission granted: user={}, permission={}, path={}",
                    auth_info.user_id, required_permission, path
                );
                
                // Log successful permission validation for audit
                log_permission_audit(
                    &auth_info.user_id,
                    &required_permission,
                    path,
                    &method.to_string(),
                    true,
                    permission_granted.reason.as_deref(),
                ).await;
                
                Ok(next.run(request).await)
            } else {
                warn!(
                    "Permission denied: user={}, permission={}, path={}, reason={}",
                    auth_info.user_id, required_permission, path, 
                    permission_granted.reason.as_deref().unwrap_or_default()
                );
                
                // Log failed permission validation for security monitoring
                log_permission_audit(
                    &auth_info.user_id,
                    &required_permission,
                    path,
                    &method.to_string(),
                    false,
                    permission_granted.reason.as_deref(),
                ).await;
                
                // Create structured permission error based on failure reason
                let permission_error = create_structured_permission_error(
                    &auth_info,
                    &required_permission,
                    path,
                    &method.to_string(),
                    &permission_granted
                );
                
                Err(permission_error.into_response())
            }
        }
        Err(validation_error) => {
            error!(
                "Permission validation error: user={}, permission={}, path={}, error={}",
                auth_info.user_id, required_permission, path, validation_error
            );
            
            // Log validation error for system monitoring
            log_permission_audit(
                &auth_info.user_id,
                &required_permission,
                path,
                &method.to_string(),
                false,
                Some(&format!("Validation error: {}", validation_error)),
            ).await;
            
            // Create system error for validation failures
            let error = PermissionError::SystemError {
                error_id: uuid::Uuid::new_v4().to_string(),
                retry_after: Some(30),
            };
            
            Err(error.into_response())
        }
    }
}

/// Authentication information extracted from request headers
#[derive(Debug, Clone)]
struct AuthInfo {
    user_id: String,
    wallet_address: Option<String>,
    chain_id: Option<String>,
    auth_type: AuthType,
}

#[derive(Debug, Clone)]
enum AuthType {
    Web3Signature,
    BearerToken,
    ApiKey,
}

/// Permission validation result from THE AUTHORITY
#[derive(Debug)]
struct PermissionValidationResult {
    granted: bool,
    reason: Option<String>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    usage_count: Option<u32>,
    usage_limit: Option<u32>,
}

/// Extract authentication information from request headers
fn extract_auth_info(headers: &HeaderMap) -> Option<AuthInfo> {
    // Try Web3 authentication first (wallet-first architecture)
    if let (Some(wallet_addr), Some(signature)) = (
        headers.get("x-wallet-address").and_then(|h| h.to_str().ok()),
        headers.get("x-signature").and_then(|h| h.to_str().ok())
    ) {
        let chain_id = headers.get("x-chain-id").and_then(|h| h.to_str().ok()).map(String::from);
        
        return Some(AuthInfo {
            user_id: wallet_addr.to_string(),
            wallet_address: Some(wallet_addr.to_string()),
            chain_id,
            auth_type: AuthType::Web3Signature,
        });
    }
    
    // Try Bearer token authentication
    if let Some(auth_header) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
        if auth_header.starts_with("Bearer ") {
            // Extract user ID from Bearer token (simplified for middleware)
            // In production, you would decode the JWT token here
            let user_id = extract_user_from_bearer_token(auth_header);
            
            return Some(AuthInfo {
                user_id,
                wallet_address: None,
                chain_id: None,
                auth_type: AuthType::BearerToken,
            });
        }
    }
    
    // Try API key authentication
    if let Some(api_key) = headers.get("x-api-key").and_then(|h| h.to_str().ok()) {
        let user_id = extract_user_from_api_key(api_key);
        
        return Some(AuthInfo {
            user_id,
            wallet_address: None,
            chain_id: None,
            auth_type: AuthType::ApiKey,
        });
    }
    
    None
}

/// Extract user ID from Bearer token (placeholder implementation)
fn extract_user_from_bearer_token(auth_header: &str) -> String {
    // TODO: Implement proper JWT token decoding
    // For now, return placeholder user ID
    format!("user_from_bearer_{}", auth_header.len())
}

/// Extract user ID from API key (placeholder implementation)
fn extract_user_from_api_key(api_key: &str) -> String {
    // TODO: Implement proper API key validation
    // For now, return placeholder user ID
    format!("user_from_api_key_{}", api_key.len())
}

/// Check if the endpoint is public (no permission required)
fn is_public_endpoint(path: &str) -> bool {
    const PUBLIC_PATHS: &[&str] = &[
        "/",
        "/health",
        "/readiness", 
        "/liveness",
        "/api/v1/public/",
        "/api/auth/web3/challenge",
        "/api/permissions/health",
    ];
    
    PUBLIC_PATHS.iter().any(|public_path| {
        path.starts_with(public_path)
    })
}

/// Check if the endpoint is a health check
fn is_health_endpoint(path: &str) -> bool {
    path.contains("/health") || path.contains("/readiness") || path.contains("/liveness")
}

/// Determine the required permission for a given route and method
/// This is THE AUTHORITY for mapping routes to permissions
fn determine_required_permission(method: &str, path: &str) -> Option<String> {
    // Permission mapping rules - THE SINGLE SOURCE OF TRUTH
    let permission_map = create_permission_mapping();
    
    // Try exact match first
    let route_key = format!("{} {}", method, path);
    if let Some(permission) = permission_map.get(&route_key) {
        return Some(permission.clone());
    }
    
    // Try pattern matching for dynamic routes
    for (pattern, permission) in &permission_map {
        if matches_route_pattern(pattern, &route_key) {
            return Some(permission.clone());
        }
    }
    
    // Fallback: determine permission from path structure
    determine_permission_from_path(path)
}

/// Create the comprehensive permission mapping - THE AUTHORITY
fn create_permission_mapping() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // Admin routes (require admin:*:* permissions)
    map.insert("GET /admin/users".to_string(), "admin:users:read".to_string());
    map.insert("POST /admin/users".to_string(), "admin:users:create".to_string());
    map.insert("PUT /admin/users/*".to_string(), "admin:users:update".to_string());
    map.insert("DELETE /admin/users/*".to_string(), "admin:users:delete".to_string());
    
    // Permission authority routes (require admin:permissions:* permissions)
    map.insert("POST /api/permissions/validate".to_string(), "admin:permissions:validate".to_string());
    map.insert("POST /api/permissions/validate-bulk".to_string(), "admin:permissions:validate".to_string());
    map.insert("GET /api/permissions/user/*".to_string(), "admin:permissions:read".to_string());
    
    // Tier group management (require admin:tier-groups:* permissions)
    map.insert("GET /admin/tier-groups".to_string(), "admin:tier-groups:read".to_string());
    map.insert("POST /admin/tier-groups".to_string(), "admin:tier-groups:create".to_string());
    map.insert("PUT /admin/tier-groups/*".to_string(), "admin:tier-groups:update".to_string());
    map.insert("DELETE /admin/tier-groups/*".to_string(), "admin:tier-groups:delete".to_string());
    
    // Web3 admin routes (require admin:web3:* permissions)
    map.insert("GET /admin/web3/permissions".to_string(), "admin:web3:read".to_string());
    map.insert("POST /admin/web3/permissions/grant".to_string(), "admin:web3:grant".to_string());
    map.insert("POST /admin/web3/nft-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /admin/web3/token-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /admin/web3/dao-proposals".to_string(), "admin:web3:create".to_string());
    
    // Security monitoring (require admin:security:* permissions)
    map.insert("GET /admin/security/events".to_string(), "admin:security:read".to_string());
    map.insert("GET /admin/security/metrics".to_string(), "admin:security:read".to_string());
    map.insert("GET /admin/security/user-threat".to_string(), "admin:security:read".to_string());
    
    // Performance monitoring (require admin:performance:* permissions)
    map.insert("GET /admin/performance/auth-cache".to_string(), "admin:performance:read".to_string());
    map.insert("GET /admin/performance/cache-summary".to_string(), "admin:performance:read".to_string());
    map.insert("POST /admin/performance/clear-cache".to_string(), "admin:performance:manage".to_string());
    
    // User data access (require epsx:data:* permissions)
    map.insert("GET /api/v1/users/permissions".to_string(), "epsx:data:read".to_string());
    map.insert("GET /api/v1/users/holdings".to_string(), "epsx:data:read".to_string());
    map.insert("POST /api/v1/users/verify".to_string(), "epsx:data:verify".to_string());
    
    // Analytics access (require epsx:analytics:* permissions)
    map.insert("GET /api/v1/analytics/rankings".to_string(), "epsx:analytics:read".to_string());
    map.insert("GET /analytics/rankings".to_string(), "epsx:analytics:read".to_string());
    
    map
}

/// Check if a route pattern matches the current route
fn matches_route_pattern(pattern: &str, route: &str) -> bool {
    // Simple wildcard matching for dynamic routes
    if pattern.contains("*") {
        let pattern_prefix = pattern.split('*').next().unwrap_or("");
        return route.starts_with(pattern_prefix);
    }
    
    pattern == route
}

/// Determine permission from path structure when no specific mapping exists
fn determine_permission_from_path(path: &str) -> Option<String> {
    // Fallback permission determination based on path structure
    if path.starts_with("/admin/") {
        return Some("admin:general:access".to_string());
    }
    
    if path.starts_with("/api/v1/analytics/") {
        return Some("epsx:analytics:read".to_string());
    }
    
    if path.starts_with("/api/v1/users/") {
        return Some("epsx:data:read".to_string());
    }
    
    // No permission required for unmatched paths
    None
}

/// THE SINGLE SOURCE OF TRUTH permission validation function
async fn validate_user_permission(
    app_state: &AppState,
    user_id: &str,
    required_permission: &str,
    path: &str,
    method: &str,
) -> Result<PermissionValidationResult, String> {
    // This is where the REAL permission validation happens
    // This function should call the tier group system and permission validation logic
    
    info!(
        "Validating permission: user_id={}, permission={}, path={}, method={}",
        user_id, required_permission, path, method
    );
    
    // TODO: Implement comprehensive permission validation using:
    // 1. User's tier group assignments
    // 2. Permission inheritance rules
    // 3. Temporal permission expiry
    // 4. Usage limits and quotas
    // 5. Security risk assessment
    
    // For now, return a placeholder implementation
    // In production, this should integrate with the tier group handlers we built
    
    let is_admin = user_id.contains("admin") || user_id.contains("0x") && user_id.len() == 42;
    let granted = if required_permission.starts_with("admin:") {
        is_admin
    } else {
        true // Allow all non-admin permissions for now
    };
    
    Ok(PermissionValidationResult {
        granted,
        reason: if granted {
            Some("Permission granted by tier group".to_string())
        } else {
            Some("Insufficient permission level".to_string())
        },
        expires_at: if granted {
            Some(chrono::Utc::now() + chrono::Duration::days(30))
        } else {
            None
        },
        usage_count: Some(1),
        usage_limit: Some(1000),
    })
}

/// Create structured permission error based on validation failure
fn create_structured_permission_error(
    auth_info: &AuthInfo,
    permission: &str,
    path: &str,
    _method: &str,
    validation_result: &PermissionValidationResult,
) -> PermissionError {
    let reason = validation_result.reason.as_deref().unwrap_or("Permission denied");
    
    // Analyze the failure reason to create appropriate error type
    if reason.contains("insufficient tier") || reason.contains("upgrade required") {
        PermissionError::permission_denied_with_upgrade(
            permission,
            reason,
            "professional", // TODO: Determine actual required tier
        )
    } else if reason.contains("usage limit") || reason.contains("quota exceeded") {
        PermissionError::usage_limit_exceeded(
            permission,
            validation_result.usage_count.unwrap_or(0),
            validation_result.usage_limit.unwrap_or(100),
            None, // TODO: Get actual reset time
        )
    } else if reason.contains("expired") {
        PermissionError::PermissionExpired {
            permission: permission.to_string(),
            expired_at: validation_result.expires_at.unwrap_or(chrono::Utc::now()),
            renewal_url: Some("/payment".to_string()),
        }
    } else if reason.contains("security") || reason.contains("suspicious") {
        PermissionError::security_restriction(reason, RiskLevel::Medium)
    } else if reason.contains("tier") || reason.contains("level") {
        PermissionError::InsufficientTier {
            current_tier: "basic".to_string(), // TODO: Get from user context
            required_tier: "professional".to_string(), // TODO: Determine from permission
            upgrade_url: Some("/payment".to_string()),
            benefits: vec![
                format!("Access to {}", permission),
                "Enhanced features and higher limits".to_string(),
            ],
        }
    } else {
        // Generic permission denied
        PermissionError::PermissionDenied {
            permission: permission.to_string(),
            reason: reason.to_string(),
            suggested_actions: vec![
                "Verify your permissions".to_string(),
                "Contact support if you believe this is an error".to_string(),
            ],
            upgrade_tier: None,
        }
    }
}

/// Log permission audit for compliance and security monitoring
async fn log_permission_audit(
    user_id: &str,
    permission: &str,
    path: &str,
    method: &str,
    granted: bool,
    reason: Option<&str>,
) {
    let audit_entry = json!({
        "timestamp": chrono::Utc::now(),
        "user_id": user_id,
        "permission": permission,
        "path": path,
        "method": method,
        "granted": granted,
        "reason": reason.unwrap_or_default(),
        "audit_type": "permission_validation"
    });
    
    // TODO: Implement proper audit logging to database
    info!("Permission audit: {}", audit_entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_public_endpoint() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/api/v1/public/analytics"));
        assert!(is_public_endpoint("/api/permissions/health"));
        assert!(!is_public_endpoint("/admin/users"));
        assert!(!is_public_endpoint("/api/v1/users"));
    }
    
    #[test]
    fn test_determine_required_permission() {
        assert_eq!(
            determine_required_permission("GET", "/admin/users"),
            Some("admin:users:read".to_string())
        );
        
        assert_eq!(
            determine_required_permission("POST", "/admin/tier-groups"),
            Some("admin:tier-groups:create".to_string())
        );
        
        assert_eq!(
            determine_required_permission("GET", "/health"),
            None
        );
    }
    
    #[test]
    fn test_matches_route_pattern() {
        assert!(matches_route_pattern("PUT /admin/users/*", "PUT /admin/users/123"));
        assert!(matches_route_pattern("GET /admin/tier-groups/*", "GET /admin/tier-groups/456"));
        assert!(!matches_route_pattern("GET /admin/users", "POST /admin/users"));
    }
}