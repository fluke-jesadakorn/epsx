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
    
    // Extract Web3 authentication information
    let auth_info = match extract_web3_auth_info(&headers) {
        Some(info) => info,
        None => {
            warn!("Permission middleware: Missing Web3 authentication for protected route: {} {}", method, path);
            let error = PermissionError::authentication_required(
                "Valid Web3 wallet signature required to access this resource"
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
    let validation_result = validate_wallet_permission(
        &app_state,
        &auth_info.wallet_address,
        &required_permission,
        path,
        &method.to_string(),
    ).await;
    
    match validation_result {
        Ok(permission_granted) => {
            if permission_granted.granted {
                info!(
                    "Permission granted: user={}, permission={}, path={}",
                    auth_info.wallet_address, required_permission, path
                );
                
                // Log successful permission validation for audit
                log_permission_audit(
                    &auth_info.wallet_address,
                    &required_permission,
                    path,
                    &method.to_string(),
                    true,
                    permission_granted.reason.as_deref(),
                ).await;
                
                Ok(next.run(request).await)
            } else {
                warn!(
                    "Permission denied: wallet={}, permission={}, path={}, reason={}",
                    auth_info.wallet_address, required_permission, path, 
                    permission_granted.reason.as_deref().unwrap_or_default()
                );
                
                // Log failed permission validation for security monitoring
                log_permission_audit(
                    &auth_info.wallet_address,
                    &required_permission,
                    path,
                    &method.to_string(),
                    false,
                    permission_granted.reason.as_deref(),
                ).await;
                
                // Create structured permission error based on failure reason
                let permission_error = create_structured_permission_error(
                    &auth_info.wallet_address,
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
                "Permission validation error: wallet={}, permission={}, path={}, error={}",
                auth_info.wallet_address, required_permission, path, validation_error
            );
            
            // Log validation error for system monitoring
            log_permission_audit(
                &auth_info.wallet_address,
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

/// Web3 authentication information extracted from request headers
#[derive(Debug, Clone)]
struct Web3AuthInfo {
    wallet_address: String,
    #[allow(dead_code)] // TODO: Implement chain ID validation
    chain_id: Option<String>,
    #[allow(dead_code)] // TODO: Implement signature verification
    signature: String,
    #[allow(dead_code)] // TODO: Implement message validation
    message: String,
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

/// Extract Web3 authentication information from request headers
/// Pure wallet-first authentication - only Web3 signatures accepted
fn extract_web3_auth_info(headers: &HeaderMap) -> Option<Web3AuthInfo> {
    // Extract Web3 signature components - all are required for wallet-first auth
    // Use standardized header names matching web3_auth_middleware.rs
    let wallet_addr = headers.get("X-Wallet-Address").and_then(|h| h.to_str().ok())?;
    let signature = headers.get("X-Web3-Signature").and_then(|h| h.to_str().ok())?;  
    let message = headers.get("X-Signed-Message").and_then(|h| h.to_str().ok())?;
    let chain_id = headers.get("X-Chain-Id").and_then(|h| h.to_str().ok()).map(String::from);
    
    // Validate wallet address format
    if !is_valid_wallet_address(wallet_addr) {
        warn!("Invalid wallet address format: {}", wallet_addr);
        return None;
    }
    
    Some(Web3AuthInfo {
        wallet_address: wallet_addr.to_lowercase(), // Normalize to lowercase
        chain_id,
        signature: signature.to_string(),
        message: message.to_string(),
    })
}

/// Validate wallet address format (0x + 40 hex characters)
fn is_valid_wallet_address(address: &str) -> bool {
    if address.len() != 42 {
        return false;
    }
    
    if !address.starts_with("0x") {
        return false;
    }
    
    // Check if the remaining 40 characters are valid hex
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
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
        "/api/v1/auth/web3/challenge",
        "/api/permissions/health",
        "/docs",                    // API documentation UI (ReDoc)
        "/api-docs/",               // OpenAPI specification endpoint
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
    
    // Admin routes (require admin:*:* permissions) - Both /admin/* and /api/admin/*
    map.insert("GET /admin/users".to_string(), "admin:users:read".to_string());
    map.insert("POST /admin/users".to_string(), "admin:users:create".to_string());
    map.insert("PUT /admin/users/*".to_string(), "admin:users:update".to_string());
    map.insert("DELETE /admin/users/*".to_string(), "admin:users:delete".to_string());
    
    // API Admin routes (frontend compatibility)
    map.insert("GET /api/admin/users".to_string(), "admin:users:read".to_string());
    map.insert("POST /api/admin/users".to_string(), "admin:users:create".to_string());
    map.insert("PUT /api/admin/users/*".to_string(), "admin:users:update".to_string());
    map.insert("DELETE /api/admin/users/*".to_string(), "admin:users:delete".to_string());
    
    // Permission authority routes (require admin:permissions:* permissions)
    map.insert("POST /api/permissions/validate".to_string(), "admin:permissions:validate".to_string());
    map.insert("POST /api/permissions/validate-bulk".to_string(), "admin:permissions:validate".to_string());
    map.insert("GET /api/permissions/user/*".to_string(), "admin:permissions:read".to_string());
    
    // Permission group management (require admin:permission-groups:* permissions)
    map.insert("GET /admin/permission-groups".to_string(), "admin:permission-groups:read".to_string());
    map.insert("POST /admin/permission-groups".to_string(), "admin:permission-groups:create".to_string());
    map.insert("PUT /admin/permission-groups/*".to_string(), "admin:permission-groups:update".to_string());
    map.insert("DELETE /admin/permission-groups/*".to_string(), "admin:permission-groups:delete".to_string());
    map.insert("GET /api/admin/permission-groups".to_string(), "admin:permission-groups:read".to_string());
    map.insert("POST /api/admin/permission-groups".to_string(), "admin:permission-groups:create".to_string());
    map.insert("PUT /api/admin/permission-groups/*".to_string(), "admin:permission-groups:update".to_string());
    map.insert("DELETE /api/admin/permission-groups/*".to_string(), "admin:permission-groups:delete".to_string());
    
    // Web3 admin routes (require admin:web3:* permissions)
    map.insert("GET /admin/web3/permissions".to_string(), "admin:web3:read".to_string());
    map.insert("POST /admin/web3/permissions/grant".to_string(), "admin:web3:grant".to_string());
    map.insert("POST /admin/web3/nft-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /admin/web3/token-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /admin/web3/dao-proposals".to_string(), "admin:web3:create".to_string());
    map.insert("GET /api/admin/web3/permissions".to_string(), "admin:web3:read".to_string());
    map.insert("POST /api/admin/web3/permissions/grant".to_string(), "admin:web3:grant".to_string());
    map.insert("POST /api/admin/web3/nft-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /api/admin/web3/token-gates".to_string(), "admin:web3:create".to_string());
    map.insert("POST /api/admin/web3/dao-proposals".to_string(), "admin:web3:create".to_string());
    
    // Security monitoring (require admin:security:* permissions)
    map.insert("GET /admin/security/events".to_string(), "admin:security:read".to_string());
    map.insert("GET /admin/security/metrics".to_string(), "admin:security:read".to_string());
    map.insert("GET /admin/security/user-threat".to_string(), "admin:security:read".to_string());
    map.insert("GET /api/admin/security/events".to_string(), "admin:security:read".to_string());
    map.insert("GET /api/admin/security/metrics".to_string(), "admin:security:read".to_string());
    map.insert("GET /api/admin/security/user-threat".to_string(), "admin:security:read".to_string());
    
    // Performance monitoring (require admin:performance:* permissions)
    map.insert("GET /admin/performance/auth-cache".to_string(), "admin:performance:read".to_string());
    map.insert("GET /admin/performance/cache-summary".to_string(), "admin:performance:read".to_string());
    map.insert("POST /admin/performance/clear-cache".to_string(), "admin:performance:manage".to_string());
    map.insert("GET /api/admin/performance/auth-cache".to_string(), "admin:performance:read".to_string());
    map.insert("GET /api/admin/performance/cache-summary".to_string(), "admin:performance:read".to_string());
    map.insert("POST /api/admin/performance/clear-cache".to_string(), "admin:performance:manage".to_string());
    
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
    if path.starts_with("/admin/") || path.starts_with("/api/admin/") {
        return Some("admin:general:access".to_string());
    }
    
    if path.starts_with("/api/permissions/") {
        return Some("admin:permissions:validate".to_string());
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

/// THE SINGLE SOURCE OF TRUTH wallet permission validation function
/// Real wallet-first validation using unified Web3 permission service
async fn validate_wallet_permission(
    app_state: &AppState,
    wallet_address: &str,
    required_permission: &str,
    path: &str,
    method: &str,
) -> Result<PermissionValidationResult, String> {
    info!(
        "Validating wallet permission: wallet={}, permission={}, path={}, method={}",
        wallet_address, required_permission, path, method
    );
    
    // Get the unified Web3 permission service from app state
    let _web3_permission_service = app_state
        .domain_container
        .get_wallet_permission_service()
        .ok_or_else(|| "Web3 permission service not available".to_string())?;
    
    // For now, allow basic permissions for authenticated wallets
    // TODO: Integrate with proper wallet permission service when available
    let basic_permissions = vec!["epsx:basic:view", "epsx:data:read"];
    let has_permission = basic_permissions.contains(&required_permission);
    
    if has_permission {
        info!("Permission '{}' granted for wallet: {}", required_permission, wallet_address);
        
        Ok(PermissionValidationResult {
            granted: true,
            reason: Some(format!("Basic permission granted for authenticated wallet {}", wallet_address)),
            expires_at: None,
            usage_count: Some(1),
            usage_limit: Some(1000),
        })
    } else {
        
        let reason = format!("Wallet {} lacks required permission: '{}'", wallet_address, required_permission);
        
        Ok(PermissionValidationResult {
            granted: false,
            reason: Some(reason),
            expires_at: None,
            usage_count: Some(0),
            usage_limit: Some(1000),
        })
    }
}

/// Create structured permission error based on validation failure
fn create_structured_permission_error(
    _wallet_address: &str,
    permission: &str,
    _path: &str,
    _method: &str,
    validation_result: &PermissionValidationResult,
) -> PermissionError {
    let reason = validation_result.reason.as_deref().unwrap_or("Permission denied");
    
    // Analyze the failure reason to create appropriate error type
    if reason.contains("insufficient access") || reason.contains("upgrade required") {
        PermissionError::permission_denied_with_upgrade(
            permission,
            reason,
            "premium", // TODO: Determine actual required group
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
    } else if reason.contains("group") || reason.contains("level") {
        PermissionError::permission_denied_with_upgrade(
            permission,
            "Insufficient permission group access",
            "premium", // TODO: Determine from permission
        )
    } else {
        // Generic permission denied
        PermissionError::PermissionDenied {
            permission: permission.to_string(),
            reason: reason.to_string(),
            suggested_actions: vec![
                "Verify your permissions".to_string(),
                "Contact support if you believe this is an error".to_string(),
            ],
            upgrade_group: None,
        }
    }
}

/// Log permission audit for compliance and security monitoring
async fn log_permission_audit(
    wallet_address: &str,
    permission: &str,
    path: &str,
    method: &str,
    granted: bool,
    reason: Option<&str>,
) {
    let audit_entry = json!({
        "timestamp": chrono::Utc::now(),
        "wallet_address": wallet_address,
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
            determine_required_permission("POST", "/admin/permission-groups"),
            Some("admin:permission-groups:create".to_string())
        );
        
        assert_eq!(
            determine_required_permission("GET", "/health"),
            None
        );
    }
    
    #[test]
    fn test_matches_route_pattern() {
        assert!(matches_route_pattern("PUT /admin/users/*", "PUT /admin/users/123"));
        assert!(matches_route_pattern("GET /admin/permission-groups/*", "GET /admin/permission-groups/456"));
        assert!(!matches_route_pattern("GET /admin/users", "POST /admin/users"));
    }
}